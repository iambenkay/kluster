use core::arch::global_asm;

global_asm!(
    include_str!("boot.S"),
    CONST_CORE_ID_MASK = const 0b11
);

#[unsafe(no_mangle)]
fn _start_rust() -> ! {
    crate::kernel::main()
}
