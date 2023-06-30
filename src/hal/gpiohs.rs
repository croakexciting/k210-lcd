
#[inline(always)]
unsafe fn u32_set_bit(p: *mut u32, bit: bool, index: usize) {
    let mask = 1 << index;
    if bit {
        *p |= mask;
    } else {
        *p &= !mask;
    }
}

trait GpiohsAccess {
    fn peripheral() -> &'static mut k210_hal::pac::gpiohs::RegisterBlock;

    fn set_input_en(index: usize, bit: bool) {
        unsafe {
            let p = &mut Self::peripheral().input_en as *mut _ as *mut _;
            u32_set_bit(p, bit, index);
        }
    }

    fn set_output_en(index: usize, bit: bool) {
        unsafe {
            let p = &mut Self::peripheral().output_en as *mut _ as *mut _;
            u32_set_bit(p, bit, index);
        }
    }

    fn set_output_value(index: usize, bit: bool) {
        unsafe {
            let p = &mut Self::peripheral().output_val as *mut _ as *mut _;
            u32_set_bit(p, bit, index);
        }
    }
}

impl GpiohsAccess for k210_hal::pac::GPIOHS {
    fn peripheral() -> &'static mut k210_hal::pac::gpiohs::RegisterBlock {
        unsafe { &mut *(k210_hal::pac::GPIOHS::ptr() as *mut _) }
    }
}

pub struct Gpiohs {
    index: usize,
}

impl Gpiohs {
    pub fn new(index: usize) -> Self {
        Gpiohs { index }
    }

    pub fn set_output(&mut self) {
        k210_hal::pac::GPIOHS::set_output_en(self.index, true);
        k210_hal::pac::GPIOHS::set_input_en(self.index, false);
    }

    pub fn set_high(&mut self) {
        k210_hal::pac::GPIOHS::set_output_value(self.index, true);
    }

    pub fn set_low(&mut self) {
        k210_hal::pac::GPIOHS::set_output_value(self.index, false);
    }
}