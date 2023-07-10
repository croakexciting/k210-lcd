pub use k210_hal::pac::dmac::channel::ctl::SINC_A as Inc;
pub use k210_hal::pac::dmac::channel::ctl::SRC_TR_WIDTH_A as TrWidth;
pub use k210_hal::pac::dmac::channel::ctl::SRC_MSIZE_A as Msize;
pub use k210_hal::pac::dmac::channel::cfg::TT_FC_A as FlowControl;
pub use k210_hal::pac::dmac::channel::cfg::HS_SEL_SRC_A as HandshakeSrcSel;
pub use k210_hal::pac::dmac::channel::cfg::HS_SEL_DST_A as HandshakeDstSel;
pub use k210_hal::pac::dmac::channel::ctl::SMS_A as Sms;
use k210_hal::pac::DMAC;
use k210_hal::pac::SYSCTL;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum  Channel {
    Ch0 = 0,
    Ch1 = 1,
    Ch2 = 2,
    Ch3 = 3,
    Ch4 = 4,
    Ch5 = 5,
}

pub enum MemType {
    Peripheral,
    Memory,
}

fn is_memory(addr: u64) -> MemType {
    let mem_len = 6 * 1024 * 1024;
    let mem_no_cache_len = 8 * 1024 * 1024;
    if ((addr >= 0x8000_0000) && (addr < 0x8000_0000 + mem_len))
        || ((addr >= 0x40000000) && (addr < 0x40000000 + mem_no_cache_len))
        || (addr == 0x50450040)
    {
        MemType::Memory
    } else {
        MemType::Peripheral
    }
}

pub struct Dmac {}

impl Dmac {
    pub fn init(&mut self) {
        unsafe {
            let sysctl = &(*SYSCTL::ptr());
            (*sysctl).clk_en_peri.modify(|_, w| w.dma_clk_en().set_bit());

            let handler = DMAC::ptr();
            self.reset();

            self.clear_common_interrupt();
            
            self.disable();

            while (*handler).cfg.read().bits() != 0x00 {}

            (*handler).chen.modify(|_, w| {
                w.ch1_en().clear_bit().ch2_en().clear_bit().ch3_en().clear_bit().
                ch4_en().clear_bit().ch5_en().clear_bit().ch6_en().clear_bit()
            });

            for ch in &(*handler).channel {
                ch.intclear.write(|w| w.bits(0xffff_ffff));
            }

            self.enable();
        }
    }

    fn reset(&mut self) {
        unsafe {
            let handler = DMAC::ptr();
            (*handler).reset.write(|w| w.rst().set_bit());
            while (*handler).reset.read().rst().bit() {}
        }
    }

    fn clear_common_interrupt(&mut self) {
        unsafe {
            let handler = DMAC::ptr();
            (*handler).com_intclear.write(|w| w.bits(0x10f))
        }
    }

    fn channel_interrupt_clear(&mut self, ch: Channel) {
        unsafe {
            let handler = DMAC::ptr();
            (*handler).channel[ch as usize].
                intclear.
                write(|w| w.bits(0xffff_ffff));
        }
    }

    fn enable(&mut self) {
        unsafe{
            let handler = DMAC::ptr();
            (*handler).cfg.modify(|_, w| w.dmac_en().set_bit().int_en().set_bit());
        }
    }

    fn disable(&mut self) {
        unsafe{
            let handler = DMAC::ptr();
            (*handler).cfg.modify(|_, w| w.dmac_en().clear_bit().int_en().clear_bit());
        }
    }

    fn channel_enable(&mut self, ch: Channel) {
        unsafe {
            let handler = DMAC::ptr();
            use Channel::*;
            match ch {
                Ch0 => (*handler).chen.modify(
                    |_, w| w.ch1_en().set_bit().ch1_en_we().set_bit()
                ),
                Ch1 => (*handler).chen.modify(
                    |_, w| w.ch2_en().set_bit().ch2_en_we().set_bit()
                ),
                Ch2 => (*handler).chen.modify(
                    |_, w| w.ch3_en().set_bit().ch3_en_we().set_bit()
                ),
                Ch3 => (*handler).chen.modify(
                    |_, w| w.ch4_en().set_bit().ch4_en_we().set_bit()
                ),
                Ch4 => (*handler).chen.modify(
                    |_, w| w.ch5_en().set_bit().ch5_en_we().set_bit()
                ),
                Ch5 => (*handler).chen.modify(
                    |_, w| w.ch6_en().set_bit().ch6_en_we().set_bit()
                ),
            }
        }
    }

    fn channel_disable(&mut self, ch: Channel) {
        unsafe {
            let handler = DMAC::ptr();
            use Channel::*;
            match ch {
                Ch0 => (*handler).chen.modify(
                    |_, w| w.ch1_en().clear_bit().ch1_en_we().clear_bit()
                ),
                Ch1 => (*handler).chen.modify(
                    |_, w| w.ch2_en().clear_bit().ch2_en_we().clear_bit()
                ),
                Ch2 => (*handler).chen.modify(
                    |_, w| w.ch3_en().clear_bit().ch3_en_we().clear_bit()
                ),
                Ch3 => (*handler).chen.modify(
                    |_, w| w.ch4_en().clear_bit().ch4_en_we().clear_bit()
                ),
                Ch4 => (*handler).chen.modify(
                    |_, w| w.ch5_en().clear_bit().ch5_en_we().clear_bit()
                ),
                Ch5 => (*handler).chen.modify(
                    |_, w| w.ch6_en().clear_bit().ch6_en_we().clear_bit()
                ),
            }
        }
    }

    fn is_channel_idle(&mut self, ch: Channel) -> bool {
        unsafe {
            let handler = DMAC::ptr();
            ((*handler).chen.read().bits() >> ch as u8) & 0x1 == 0
        }
    }

    fn wait_idle(&mut self, ch: Channel) {
        while !self.is_channel_idle(ch) {
            
        }

        self.channel_interrupt_clear(ch);
    }

    pub fn wait_done(&mut self, ch: Channel) {
        self.wait_idle(ch);
    }

    fn set_channel_param(
        &mut self,
        ch: Channel,
        src: u64,
        dst: u64,
        src_inc: Inc,
        dest_inc: Inc,
        trans_width: TrWidth,
        burst_size: Msize,
        block_size: u32,
    ) {
        unsafe {
            let handler = DMAC::ptr();
            let chr = &(*handler).channel[ch as usize];
            let mem_type = (is_memory(src), is_memory(dst));

            let flow_control = {
                use MemType::*;

                match mem_type {
                    (Memory, Memory) => FlowControl::MEM2MEM_DMA,
                    (Memory, Peripheral) => FlowControl::MEM2PRF_DMA,
                    (Peripheral, Memory) => FlowControl::PRF2MEM_DMA,
                    (Peripheral, Peripheral) => FlowControl::PRF2PRF_DMA,
                }
            };

            chr.cfg.modify(
                |_, w| 
                w.tt_fc().variant(flow_control).
                hs_sel_src().variant(match mem_type.0 {
                    MemType::Memory => HandshakeSrcSel::SOFTWARE,
                    MemType::Peripheral => HandshakeSrcSel::HARDWARE,
                }).hs_sel_dst().variant(match mem_type.1 {
                    MemType::Memory => HandshakeDstSel::SOFTWARE,
                    MemType::Peripheral => HandshakeDstSel::HARDWARE,
                }).src_per().bits(ch as u8).dst_per().bits(ch as u8).
                src_multblk_type().bits(0x00).dst_multblk_type().bits(0x00)
            );

            chr.sar.write(|w| w.bits(src));
            chr.dar.write(|w| w.bits(dst));

            chr.ctl.modify(|_, w| {
                w.sms().variant(Sms::AXI_MASTER_1).dms().variant(Sms::AXI_MASTER_2).
                sinc().variant(src_inc).dinc().variant(dest_inc).
                src_tr_width().variant(trans_width).dst_tr_width().variant(trans_width).
                src_msize().variant(burst_size).dst_msize().variant(burst_size)
            });

            chr.block_ts.write(|w| w.block_ts().bits(block_size - 1));
        }
    }

    pub fn set_single_mode(
        &mut self,
        ch: Channel,
        src: u64,
        dest: u64,
        src_inc: Inc,
        dest_inc: Inc,
        trans_width: TrWidth,
        burst_size: Msize,
        block_size: u32,
    ) {
        self.channel_interrupt_clear(ch);
        self.channel_disable(ch);
        self.wait_idle(ch);
        self.set_channel_param(
            ch, src, dest, src_inc, dest_inc, 
            trans_width, burst_size, block_size
        );
        self.enable();
        self.channel_enable(ch);
    }
}