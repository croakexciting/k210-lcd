#![no_std]

pub mod hal;

use hal::gpiohs;
use hal::spi::{Spi, WORK_MODE_A, FRAME_FORMAT_A, AITM_A};

use core::convert::Into;
use core::marker::Copy;

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
    pub spi: Spi,
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

        // spi initial
        let mut spi = Spi::new_spi0(WORK_MODE_A::MODE0, FRAME_FORMAT_A::OCTAL, 32, 1, 8_000_000);
        spi.set_non_standard_mode(0, 32, 0, AITM_A::AS_FRAME_FORMAT);

        Lcd { spi, rst, dcx }
    }

    pub fn send_command(&mut self, cmd: u8) {
        self.dcx.set_low();
        let buf = [cmd];
        self.spi.send_data(CHIP_SELECT, &buf);
    }

    pub fn send_data<U: Into<u32> + Copy> (&mut self, data: &[U]) {
        self.dcx.set_high();
        self.spi.send_data(CHIP_SELECT, data)
    }

    pub fn fill_data(&mut self, value: u32, len: usize) {
        self.dcx.set_high();
        self.spi.fill_data(CHIP_SELECT, value, len);
    }
}
