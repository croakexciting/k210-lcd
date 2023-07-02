#![allow(warnings)]
#![no_std]
#![no_main]

use k210_lcd::{
    Lcd, usleep, 
    k210_hal::{
        self, 
        pac::sysctl, 
        sysctl::SysctlExt, 
        prelude::*,
        stdout::Stdout
    }
};

use panic_halt as _;

#[riscv_rt::entry]
fn main() -> ! {
    let p = k210_hal::pac::Peripherals::take().unwrap();
    let mut sysctl = p.SYSCTL.constrain();
    let fpioa = p.FPIOA.split(&mut sysctl.apb0);
    let _dcx = fpioa.io38.into_function(k210_hal::fpioa::GPIOHS22);
    let _rst = fpioa.io37.into_function(k210_hal::fpioa::GPIOHS21);
    let _ss = fpioa.io36.into_function(k210_hal::fpioa::SPI0_SS3);
    let _clk = fpioa.io39.into_function(k210_hal::fpioa::SPI0_SCLK);

    let mut lcd = Lcd::new_lcd();
    lcd.send_command(0x1);
    usleep(50_000);
    lcd.send_command(0x11);
    lcd.send_command(0x51);
    lcd.send_byte(0xFF);
    lcd.send_command(0x13);
    lcd.send_command(0x29);

    usleep(1_000_000);

    // lcd.send_command(0x28);

    // lcd.fill_rectangle(0, 0, 100, 100, 0xFFFF);
    // let mut i = 0;
    loop {
        lcd.send_command(0x28);
        usleep(50_000);
        // lcd.send_command(0x29);
        // usleep(50_000);
        // if i < 240 {
        //     lcd.send_command(0x28);
        //     lcd.draw_point(i, i, 0x0);
        //     i += 1;
        //     usleep(100_000);
        // }
    }
}