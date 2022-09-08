#![allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]

#[doc(hidden)]
#[macro_export]
macro_rules! __cmacro__MEMORY_BARRIER {
    () => {
        ::std::arch::asm!(clobber_abi("C"), options(preserves_flags),)
    };
}
pub use __cmacro__MEMORY_BARRIER as MEMORY_BARRIER;
