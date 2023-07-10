#![allow(warnings)]
#![no_std]
#![no_main]

use k210_lcd::{
    Lcd, usleep, constant::*,
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

    let mut lcd = Lcd::new_lcd(320, 240, Dir::YxLrud);
    lcd.lcd_draw_string(120, 116, "Hello CWYZ", BLUE);

    loop {}
}