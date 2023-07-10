#![allow(warnings)]
#![no_std]
#![no_main]

use k210_lcd::{
    Lcd, usleep,
    hal::gpiohs::Gpiohs,
    k210_hal::{
        self, 
        pac::sysctl, 
        sysctl::SysctlExt, 
        prelude::*,
        stdout::Stdout,
        gpio::Gpio
    }
};

use panic_halt as _;

use buddy_system_allocator::LockedHeap;

#[global_allocator]
/// heap allocator instance
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();


fn init_heap() {
    extern "C" {
        fn _sheap();
        fn _heap_size();
    }

    unsafe {
        HEAP_ALLOCATOR.lock().init(_sheap as usize, _heap_size as usize);
    }
}

#[riscv_rt::entry]
fn main() -> ! {
    init_heap();
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
    usleep(120_000);
    lcd.send_command(0x3A);
    lcd.send_byte(0x55);
    usleep(10_000);
    lcd.send_command(0x13);
    usleep(10_000);
    lcd.send_command(0x29);
    usleep(10_000);

    usleep(100_000);
    lcd.fill_rectangle(0, 20, 120, 100, 0);

    loop {}
}