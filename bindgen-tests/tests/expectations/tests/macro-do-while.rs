#![allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]

#[allow(non_snake_case, unused_mut)]
#[inline(always)]
pub unsafe extern "C" fn vTaskDelayUntil(
    mut pxPreviousWakeTime: ::std::os::raw::c_int,
    mut xTimeIncrement: ::std::os::raw::c_int,
) {
    loop {
        {
            drop(xTaskDelayUntil(pxPreviousWakeTime, xTimeIncrement))
        };
        if 0 == Default::default() {
            break;
        }
    }
}
extern "C" {
    pub fn xTaskDelayUntil(
        arg1: ::std::os::raw::c_int,
        arg2: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
