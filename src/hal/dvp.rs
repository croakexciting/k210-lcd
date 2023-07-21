use k210_hal::{
    pac::SYSCTL,
    sysctl::ACLK,
    pac::dvp::axi::GM_MLEN_A::{BYTE1, BYTE4},
    pac::dvp::sccb_cfg::BYTE_NUM_A::{NUM2, NUM3},
    pac::DVP,
};

use super::utils::usleep;

pub struct Dvp {
    pub sccb_reg_len: u8
}

impl Dvp {
    pub fn new_dvp(
        sccb_reg_len: u8,
    ) -> Self {
        unsafe {
            // enable bus clock and device clock
            let sysptr = SYSCTL::ptr();            
            (*sysptr).clk_en_cent.modify(|_, w| w.apb1_clk_en().set_bit());
            (*sysptr).clk_en_peri.modify(|_, w| w.dvp_clk_en().set_bit());
            (*sysptr).peri_reset.modify(|_, w| w.dvp_reset().set_bit());
            usleep(10);
            (*sysptr).peri_reset.modify(|_, w| w.dvp_reset().clear_bit());
            (*sysptr).misc.write(|w| w.spi_dvp_data_enable().set_bit());
            (*sysptr).power_sel.modify(|_, w| w.power_mode_sel6().clear_bit());
            (*sysptr).power_sel.modify(|_, w| w.power_mode_sel7().clear_bit());

            let handler = DVP::ptr();
            (*handler).cmos_cfg.modify(|_, w| w.clk_div().bits(3).clk_enable().set_bit());
            (*handler).sccb_cfg.modify(|_, w| w.scl_lcnt().bits(255).scl_hcnt().bits(255));
            let dvp = Dvp { sccb_reg_len };
            dvp
        }
    }

    pub fn set_xclk(&mut self, xclk: u32) {
        unsafe {
            let aclk = ACLK::steal();
            let aclk_freq = aclk.get_frequency().0 as u64;
            let sysptr = SYSCTL::ptr();
            let apb1_clk_sel = (*sysptr).clk_sel0.read().apb1_clk_sel().bits();
            let apb1_clk = aclk_freq as u32 / (apb1_clk_sel as u32 + 1);

            let mut period: u32 = 0;
            if apb1_clk > xclk * 2 {
                period = (apb1_clk / (xclk * 2)) - 1;
            }

            if period > 255 {
                period = 255;
            }

            let handler = DVP::ptr();
            (*handler).cmos_cfg.modify(|_, w| w.clk_div().bits(period as u8).clk_enable().set_bit());
        }
    }

    pub fn enable_burst(&mut self) {
        unsafe {
            let handler = DVP::ptr();
            (*handler).dvp_cfg.modify(|_, w| w.burst_size_4beats().set_bit());
            (*handler).axi.modify(|_, w| w.gm_mlen().variant(BYTE4))
        }
    }

    pub fn disable_burst(&mut self) {
        unsafe {
            let handler = DVP::ptr();
            (*handler).dvp_cfg.modify(|_, w| w.burst_size_4beats().clear_bit());
            (*handler).axi.modify(|_, w| w.gm_mlen().variant(BYTE1));
        }
    }

    pub fn set_image_format(&mut self, format: u8) {
        unsafe {
            let handler = DVP::ptr();
            (*handler).dvp_cfg.modify(|_, w| w.format().bits(format));
        }
    }

    pub fn set_image_size(&mut self, width: u32, height: u32) {
        unsafe {
            let handler = DVP::ptr();
            let burst_size = (*handler).dvp_cfg.read().burst_size_4beats().bit();
            if burst_size {
                (*handler).dvp_cfg.modify(|_, w| w.href_burst_num().bits((width / 8 / 4) as u8));
            } else {
                (*handler).dvp_cfg.modify(|_, w| w.href_burst_num().bits((width / 8 / 1) as u8));
            }

            (*handler).dvp_cfg.modify(|_, w| w.line_num().bits(height as u16));
        }
    }

    pub fn set_ai_output_enable(&mut self, b: bool) {
        unsafe {
            let handler = DVP::ptr();
            if b {
                (*handler).dvp_cfg.modify(|_, w| w.ai_output_enable().set_bit());
            } else {
                (*handler).dvp_cfg.modify(|_, w| w.ai_output_enable().clear_bit());
            }
        }
    }

    pub fn set_display_output_enable(&mut self, b: bool) {
        unsafe {
            let handler = DVP::ptr();
            (*handler).dvp_cfg.modify(|_, w| w.display_output_enable().bit(b));
        }
    }

    pub fn set_dvp_interrupt(&mut self, b: bool) {
        unsafe {
            let handler = DVP::ptr();
            (*handler).dvp_cfg.modify(|_, w| w.start_int_enable().bit(b));
            (*handler).dvp_cfg.modify(|_, w| w.finish_int_enable().bit(b));
        }
    }

    pub fn clear_interrupt_status(&mut self) {
        unsafe {
            let handler = DVP::ptr();
            (*handler).sts.modify(|_, w| w.frame_start().set_bit().frame_start_we().set_bit());
            (*handler).sts.modify(|_, w| w.frame_finish().set_bit().frame_finish_we().set_bit());
        }
    }

    pub fn set_display_addr(&mut self, addr: u32) {
        unsafe {
            let handler = DVP::ptr();
            (*handler).rgb_addr.write(|w| w.bits(addr));
        }
    }

    pub fn set_auto(&mut self, b: bool) {
        unsafe {
            let handler = DVP::ptr();
            (*handler).dvp_cfg.modify(|_, w| w.auto_enable().bit(b));
        }
    }

    pub fn start_convert(&mut self) {
        unsafe {
            let handler = DVP::ptr();
            (*handler).sts.write(|w| w.dvp_en().set_bit().dvp_en_we().set_bit());
        }
    }

    pub fn get_image(&mut self) {
        unsafe {
            let handler = DVP::ptr();
            while !(*handler).sts.read().frame_start().bit() {}
            (*handler).sts.write(|w| w.frame_start().set_bit().frame_start_we().set_bit());
            while !(*handler).sts.read().frame_start().bit() {}
            (*handler).sts.write(
                |w| 
                w.frame_finish().set_bit().
                frame_finish_we().set_bit().
                frame_start().set_bit().
                frame_start_we().set_bit().
                dvp_en().set_bit().
                dvp_en_we().set_bit()
            );
            while !(*handler).sts.read().frame_finish().bit() {}
        }
    }

    pub fn sccb_send(&mut self, dev_addr: u8, reg_addr: u8, reg_data: u8) {
        unsafe {
            let handler = DVP::ptr();
            (*handler).sccb_cfg.modify(|_, w| w.byte_num().variant(NUM3));
            (*handler).sccb_ctl.write(|w| w.device_address().bits(dev_addr | 0x1).reg_address().bits(reg_addr).wdata_byte0().bits(reg_data));
            self.sccb_start_transfer();
        }
    }

    pub fn sccb_query(&mut self, dev_addr: u8, reg_addr: u8) -> u8 {
        unsafe {
            let handler = DVP::ptr();
            (*handler).sccb_cfg.modify(|_, w| w.byte_num().variant(NUM2));
            (*handler).sccb_ctl.write(|w| w.device_address().bits(dev_addr | 0x1).reg_address().bits(reg_addr));
            self.sccb_start_transfer();
            (*handler).sccb_ctl.write(|w| w.device_address().bits(dev_addr));
            self.sccb_start_transfer();
            (*handler).sccb_cfg.read().rdata().bits()
        }
    }

    fn sccb_start_transfer(&mut self) {
        unsafe {
            let handler = DVP::ptr();
            while (*handler).sts.read().sccb_en().bit() {};
            (*handler).sts.write(|w| w.sccb_en().set_bit().sccb_en_we().set_bit());
            while (*handler).sts.read().sccb_en().bit() {};
        }
        usleep(1000);
    }
}