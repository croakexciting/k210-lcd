    pub use k210_hal::pac::sysctl::dma_sel0::DMA_SEL0_A as DmaSelect;
    use k210_hal::pac::SYSCTL;
    use crate::hal::dmac::Channel;

    pub(crate) fn set_dma_sel(ch: Channel, sel: DmaSelect) {
        unsafe { 
            let ptr = &*(SYSCTL::ptr()); 
            match ch {
                Channel::Ch0 => ptr.dma_sel0.modify(|_, w| w.dma_sel0().variant(sel)),
                Channel::Ch1 => ptr.dma_sel0.modify(|_, w| w.dma_sel1().variant(sel)),
                Channel::Ch2 => ptr.dma_sel0.modify(|_, w| w.dma_sel2().variant(sel)),
                Channel::Ch3 => ptr.dma_sel0.modify(|_, w| w.dma_sel3().variant(sel)),
                Channel::Ch4 => ptr.dma_sel0.modify(|_, w| w.dma_sel4().variant(sel)),
                Channel::Ch5 => ptr.dma_sel1.modify(|_, w| w.dma_sel5().variant(sel)),
            }
        }
    }