#![no_std]

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
pub use k210_hal;
pub mod hal;

use hal::gpiohs;
use hal::spi::{Spi, WORK_MODE_A, FRAME_FORMAT_A, AITM_A};
use hal::dmac::{Dmac, Channel};

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
}

impl Lcd {
    pub fn new_lcd() -> Self {
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

        Lcd { rst, dcx }
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

    pub fn send_bytes(&mut self, data: &Vec<u32>) {
        self.dcx.set_high();
        let mut spi = Spi::new_spi0(WORK_MODE_A::MODE0, FRAME_FORMAT_A::OCTAL, 8, 0, 10_000_000);
        spi.set_non_standard_mode(0, 8, 0, AITM_A::AS_FRAME_FORMAT);
        spi.send_data_dma(CHIP_SELECT, data.as_ptr() as u64, data.len() as u32, Channel::Ch0);
    }

    pub fn send_shorts(&mut self, data: &Vec<u32>) {
        self.dcx.set_high();
        let mut spi = Spi::new_spi0(WORK_MODE_A::MODE0, FRAME_FORMAT_A::OCTAL, 16, 0, 10_000_000);
        spi.set_non_standard_mode(0, 16, 0, AITM_A::AS_FRAME_FORMAT);
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
        let mut buf: Vec<u32> = vec![0; 4];
        buf[0] = ((x1 >> 8) & 0x00ff).into();
        buf[1] = (x1 & 0x00ff).into();
        buf[2] = ((x2 >> 8) & 0x00ff).into();
        buf[3] = (x2 & 0x00ff).into();
        self.send_command(0x2A);
        self.send_bytes(&buf);

        buf[0] = ((y1 >> 8) & 0x00ff).into();
        buf[1] = (y1 & 0x00ff).into();
        buf[2] = ((y2 >> 8) & 0x00ff).into();
        buf[3] = (y2 & 0x00ff).into();
        self.send_command(0x2B);
        self.send_bytes(&buf);

        self.send_command(0x2C);
    }

    pub fn fill_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: u16) {
        if x1 == x2 || y1 == y2 {return};
        self.lcd_set_area(x1, y1, x2-1, y2-1);
        self.fill_data(color as u32, ((x2 - x1) * (y2 - y1) / 2) as usize);
    }

    // pub fn draw_point(&mut self, x: u16, y: u16, color: u16) {
    //     self.lcd_set_area(x, y, x, y);
    //     let mut buf: [u8; 2] = [0; 2];
    //     buf[0] = (color >> 8) as u8;
    //     buf[1] = color as u8;
    //     self.send_bytes(&buf);
    // }

    // pub fn lcd_clear(&mut self) {
    //     self.lcd_set_area(0, 100, 160-1, 220-1);
    //     let buf: [u32; 9600] = [0; 9600];
    //     // self.fill_data(0xffff_ffff, 240 * 240 / 2);
    //     self.send_words(&buf);
    // }
    
}
