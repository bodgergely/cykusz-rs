use kernel::mm::VirtAddr;

pub fn init(stack_top: VirtAddr) {
    ::arch::tls::init(stack_top);
}

pub fn is_ready() -> bool {
    ::kernel::smp::is_smp_initialised()
}
