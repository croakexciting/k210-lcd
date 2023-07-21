#![allow(warnings)]
#![no_std]
#![no_main]

use k210_lcd::{
    lcd::Lcd,
    camera::Camera,
    hal::utils::usleep, 
    constant::*,
    hal::gpiohs::Gpiohs,
    k210_hal::{
        self, 
        pac::sysctl, 
        sysctl::SysctlExt, 
        prelude::*,
        stdout::Stdout,
        gpio::Gpio, spi::FrameFormat
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

    fpioa.io40.into_function(k210_hal::fpioa::SCCB_SDA);
    fpioa.io41.into_function(k210_hal::fpioa::SCCB_SCLK);
    fpioa.io42.into_function(k210_hal::fpioa::GPIOHS1);
    fpioa.io43.into_function(k210_hal::fpioa::CMOS_VSYNC);
    fpioa.io44.into_function(k210_hal::fpioa::GPIOHS2);
    fpioa.io45.into_function(k210_hal::fpioa::CMOS_HREF);
    fpioa.io46.into_function(k210_hal::fpioa::CMOS_XCLK);
    fpioa.io47.into_function(k210_hal::fpioa::CMOS_PCLK);

    let clocks = k210_hal::clock::Clocks::new();
    let _uarths_tx = fpioa.io5.into_function(k210_hal::fpioa::UARTHS_TX);

    // Configure UART
    let serial = p.UARTHS.configure(
        115_200.bps(), 
        &clocks
    );
    let (mut tx, _) = serial.split();

    // todo: new stdout design (simple Write impl?)
    let mut stdout = Stdout(&mut tx);

    writeln!(stdout, "Hello, Rust!").ok();
    // let _blue = fpioa.io8.into_function(k210_hal::fpioa::GPIOHS1);
    // let mut blue = Gpiohs::new(1);
    // blue.set_output();

    let mut lcd = Lcd::new_lcd(320, 240, Dir::YxRlud);
    lcd.lcd_draw_string(120, 116, "Hello CWYZ", BLUE);

    let mut dvp = Camera::new_camera();

    writeln!(stdout, "display out addr at {}", dvp.buf.as_ptr() as u32);

    let mut i = 0;

    loop {
        dvp.dvp.get_image();
        lcd.lcd_draw_picture(0, 0, 320, 240, &dvp.buf);
    }
}