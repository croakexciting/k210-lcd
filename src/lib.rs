#![no_std]

pub mod hal;
pub mod constant;
extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use alloc::str;
pub use k210_hal;

use hal::gpiohs;
use hal::spi::{Spi, WORK_MODE_A, FRAME_FORMAT_A, AITM_A};
use hal::dmac::{Dmac, Channel};
use crate::constant::*;

const CPU_FREQ: usize = 390_000_000;

pub fn usleep(usec: usize) {
    let interval = usec * CPU_FREQ/1_000_000;
    let current = riscv::register::mcycle::read();
    loop {
        if (riscv::register::mcycle::read() - current) > interval {
            break;
        }
    }
}

pub fn sleep(sec: usize) {
    usleep(sec * 1_000_000);
}

const RST: usize = 21;
const DCX: usize = 22;

const CHIP_SELECT: u32 = 3;

pub struct Lcd {
    pub rst: gpiohs::Gpiohs,
    pub dcx: gpiohs::Gpiohs,
    pub max_x: u16,
    pub max_y: u16,
}

impl Lcd {
    pub fn new_lcd(max_x: u16, max_y: u16, dir: Dir) -> Self {
        // gpiohs initial
        let mut rst = gpiohs::Gpiohs::new(RST);
        let mut dcx = gpiohs::Gpiohs::new(DCX);
        rst.set_output();
        dcx.set_output();

        // reset lcd
        rst.set_high();
        usleep(50_000);
        rst.set_low();
        usleep(50_000);
        rst.set_high();

        let mut dma = Dmac {};
        dma.init();

        let mut lcd = Lcd { rst, dcx, max_x, max_y };
        
        lcd.send_command(0x1);
        usleep(50_000);
        lcd.send_command(0x11);
        usleep(50_000);
        lcd.send_command(0x3A);
        lcd.send_byte(0x55);
        usleep(10_000);
        lcd.send_command(0x21);
        usleep(10_000);
        lcd.send_command(0x36);
        lcd.send_byte(dir as u8);
        usleep(10_000);
        lcd.send_command(0x29);
        usleep(10_000);
        lcd.fill_rectangle(0, 0, max_x, max_y, WHITE);
        
        lcd
    }

    pub fn send_command(&mut self, cmd: u8) {
        self.dcx.set_low();
        let buf = vec![cmd as u32];
        let mut spi = Spi::new_spi0(WORK_MODE_A::MODE0, FRAME_FORMAT_A::OCTAL, 8, 0, 10_000_000);
        spi.set_non_standard_mode(8, 0, 0, AITM_A::AS_FRAME_FORMAT);
        // spi.send_data(CHIP_SELECT, &buf);
        spi.send_data_dma(CHIP_SELECT, buf.as_ptr() as u64, buf.len() as u32, Channel::Ch0);
    }

    pub fn send_byte(&mut self, cmd: u8) {
        self.dcx.set_high();
        let buf = vec![cmd as u32];
        let mut spi = Spi::new_spi0(WORK_MODE_A::MODE0, FRAME_FORMAT_A::OCTAL, 8, 0, 10_000_000);
        spi.set_non_standard_mode(0, 8, 0, AITM_A::AS_FRAME_FORMAT);
        spi.send_data_dma(CHIP_SELECT, buf.as_ptr() as u64, buf.len() as u32, Channel::Ch0);
    }

    pub fn send_bytes(&mut self, data: &Vec<u8>) {
        self.dcx.set_high();
        let mut spi = Spi::new_spi0(WORK_MODE_A::MODE0, FRAME_FORMAT_A::OCTAL, 8, 0, 10_000_000);
        spi.set_non_standard_mode(0, 8, 0, AITM_A::AS_FRAME_FORMAT);
        
        let mut buf = vec![0; data.len()];
        for i in 0..data.len() {
            buf[i] = data[i] as u32;
        }
        spi.send_data_dma(CHIP_SELECT, buf.as_ptr() as u64, buf.len() as u32, Channel::Ch0);
    }

    pub fn send_shorts(&mut self, data: &Vec<u16>) {
        self.dcx.set_high();
        let mut spi = Spi::new_spi0(WORK_MODE_A::MODE0, FRAME_FORMAT_A::OCTAL, 16, 0, 10_000_000);
        spi.set_non_standard_mode(0, 16, 0, AITM_A::AS_FRAME_FORMAT);
        
        let mut buf = vec![0; data.len()];
        for i in 0..data.len() {
            buf[i] = data[i] as u32;
        }
        spi.send_data_dma(CHIP_SELECT, data.as_ptr() as u64, data.len() as u32, Channel::Ch0);
    }

    pub fn send_words(&mut self, data: &Vec<u32>) {
        self.dcx.set_high();
        let mut spi = Spi::new_spi0(WORK_MODE_A::MODE0, FRAME_FORMAT_A::OCTAL, 32, 0, 10_000_000);
        spi.set_non_standard_mode(0, 32, 0, AITM_A::AS_FRAME_FORMAT);
        spi.send_data_dma(CHIP_SELECT, data.as_ptr() as u64, data.len() as u32, Channel::Ch0);
    }

    pub fn fill_data(&mut self, value: u32, len: usize) {
        self.dcx.set_high();
        let mut spi = Spi::new_spi0(WORK_MODE_A::MODE0, FRAME_FORMAT_A::OCTAL, 32, 0, 10_000_000);
        spi.set_non_standard_mode(0, 32, 0, AITM_A::AS_FRAME_FORMAT);
        let buf = vec![value; len];
        spi.send_data_dma(CHIP_SELECT, buf.as_ptr() as u64, len as u32, Channel::Ch0);
    }

    pub fn lcd_set_area(&mut self, x1: u16, y1: u16, x2: u16, y2: u16) {
        let mut buf: Vec<u8> = vec![0; 4];
        buf[0] = (x1 >> 8) as u8;
        buf[1] = x1 as u8;
        buf[2] = (x2 >> 8) as u8;
        buf[3] = x2 as u8;
        self.send_command(0x2A);
        self.send_bytes(&buf);

        buf[0] = (y1 >> 8) as u8;
        buf[1] = y1 as u8;
        buf[2] = (y2 >> 8) as u8;
        buf[3] = y2 as u8;
        self.send_command(0x2B);
        self.send_bytes(&buf);

        self.send_command(0x2C);
    }

    pub fn fill_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: u16) {
        if x1 == x2 || y1 == y2 {return};
        self.lcd_set_area(x1, y1, x2-1, y2-1);
        let color = (color as u32) << 16 | color as u32;
        self.fill_data(color as u32, (((x2 - x1) as u32) * ((y2 - y1) as u32) / 2) as usize);
    }

    pub fn lcd_draw_point(&mut self, x: u16, y: u16, color: u16) {
        self.lcd_set_area(x, y, x, y);
        let buf = vec![(color >> 8) as u8, color as u8];
        self.send_bytes(&buf)
    }

    pub fn lcd_draw_char(&mut self, x: u16, y: u16, c: char, color: u16) {
        let mut my = y;
        for i in 0..16 {
            let index = (c as usize) * 16 + i;
            let mut data = ASCII0816[index];
            for j in 0..8 {
                if data & 0x80 > 0 {
                    self.lcd_draw_point(x + j, my, color);
                }
                data = data << 1;
            }
            my += 1;
        }
    }

    pub fn lcd_draw_string(&mut self, x: u16, y: u16, s: &str, color: u16) {
        let mut mx = x;
        for &c in s.as_bytes() {
            self.lcd_draw_char(mx, y, c as char, color);
            mx = mx + 8;
        }
    }
    
}
