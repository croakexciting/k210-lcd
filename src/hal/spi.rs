use k210_hal::{
    pac::SPI0,
    pac::spi0::ctrlr0::TMOD_A,
    pac::SYSCTL,
    sysctl::ACLK,
};

pub use k210_hal::{
    pac::spi0::ctrlr0::WORK_MODE_A,
    pac::spi0::ctrlr0::FRAME_FORMAT_A,
    pac::spi0::spi_ctrlr0::AITM_A,
};

use core::convert::Into;
use core::marker::Copy;
pub struct Spi {}

impl Spi {
    pub fn new_spi0(
        mode: WORK_MODE_A,
        frame_format: FRAME_FORMAT_A,
        data_bit_length: u8,
        endian: u8,
        baud: u32,
    ) -> Self {
        unsafe {
            // spi0 don't receive data
            let tmod = TMOD_A::TRANS;
            let handler = SPI0::ptr();
            // no interrupts for now, we just send data
            (*handler).imr.write(|w| w.bits(0x00));
            // TODO: dma support
            (*handler).dmacr.write(|w| w.bits(0x00));
            (*handler).dmatdlr.write(|w| w.bits(0x10));
            (*handler).dmardlr.write(|w| w.bits(0x00));
            
            (*handler).ser.write(|w| w.bits(0x00));
            (*handler).ssienr.write(|w| w.bits(0x00));

            (*handler).ctrlr0.write(|w| {
                w.work_mode().variant(mode).
                tmod().variant(tmod).
                frame_format().variant(frame_format).
                data_length().bits(data_bit_length - 1)
            });
            (*handler).spi_ctrlr0.reset();
            (*handler).endian.write(|w| w.bits(endian as u32));

            // enable bus clock and device clock
            let ptr = SYSCTL::ptr();
            (*ptr).clk_en_cent.modify(|_r, w| w.apb2_clk_en().set_bit());
            (*ptr).clk_en_peri.modify(|_r, w| w.spi0_clk_en().set_bit());
            (*ptr).clk_th1.modify(|_r, w| w.spi0_clk().bits(0));

            // enable spi peripheral
            (*ptr).misc.write(|w| w.spi_dvp_data_enable().set_bit());

            // set baudrate
            let aclk = ACLK::steal();
            let aclk_freq = aclk.get_frequency().0 as u64;
            let apb2_clk_sel = (*ptr).clk_sel0.read().apb2_clk_sel().bits();
            let mut b = (aclk_freq as u32 / (apb2_clk_sel as u32 + 1)) / baud;
            if b < 2 {
                b = 2;
            } else if b > 65534 {
                b = 65534;
            }

            (*handler).baudr.write(|w| w.bits(b));

            Spi {}
        }
    }

    pub fn set_non_standard_mode(
        &mut self, instruction_len: u32, addr_len: u32,
        wait_cycles: u8, aitm: AITM_A
    ) {
        let instl_r = match instruction_len {
            0 => 0,
            4 => 1,
            8 => 2,
            16 => 3,
            _ => 100,
        };

        let addrl_r = (addr_len / 4) as u8;

        unsafe {
            let handler = SPI0::ptr();
            (*handler).spi_ctrlr0.write(|w| {
                w.aitm().variant(aitm).
                wait_cycles().bits(wait_cycles).
                inst_length().bits(instl_r).
                addr_length().bits(addrl_r)
            });
        }
    }

    pub fn send_data<U: Into<u32> + Copy>(&mut self, cs: u32, tx: &[U]) {
        unsafe {
            let handler = SPI0::ptr();
            (*handler).ser.write(|w| w.bits(1 << cs));
            (*handler).ssienr.write(|w| w.bits(0x01));

            let mut room = 0;
            for &val in tx {
                while room == 0 {
                    room = 32 - (*handler).txflr.read().bits();
                }
                (*handler).dr[0].write(|w| w.bits(val.into()));
                room -= 1;
            }

            while ((*handler).sr.read().bits() & 0x05) != 0x04 {
                // IDLE
            }

            (*handler).ser.write(|w| w.bits(0x00));
            (*handler).ssienr.write(|w| w.bits(0x00));
        }
    }

    pub fn fill_data(&mut self, cs: u32, value: u32, len: usize) {
        unsafe {
            let handler = SPI0::ptr();
            (*handler).ser.write(|w| w.bits(1 << cs));
            (*handler).ssienr.write(|w| w.bits(0x01));

            let mut room = 0;
            let mut l = len;
            while l != 0 {
                while room == 0 {
                    room = 32 - (*handler).txflr.read().bits();
                }
                (*handler).dr[0].write(|w| w.bits(value));
                room -= 1;
                l -= 1;
            }

            while ((*handler).sr.read().bits() & 0x05) != 0x04 {
                // IDLE
            }

            (*handler).ser.write(|w| w.bits(0x00));
            (*handler).ssienr.write(|w| w.bits(0x00));
        }
    }
}
