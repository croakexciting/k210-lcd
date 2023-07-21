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