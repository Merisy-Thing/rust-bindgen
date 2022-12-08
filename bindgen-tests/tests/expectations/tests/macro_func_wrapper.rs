#![allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]

pub const Y: u32 = 7;
#[allow(non_snake_case, unused_mut)]
#[inline(always)]
pub unsafe extern "C" fn wrapper_func(mut x: ::std::os::raw::c_int) {
    func(x, 7);
}
extern "C" {
    pub fn func(arg1: ::std::os::raw::c_int, arg2: ::std::os::raw::c_int);
}
