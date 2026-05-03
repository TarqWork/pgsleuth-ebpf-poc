#![no_std]
#![no_main]

// Step 4: Minimal kprobe attached to do_sys_openat2
// No maps or logging to avoid legacy map definition issues

use aya_ebpf::{
    macros::kprobe,
    programs::ProbeContext,
};

#[kprobe(function = "do_sys_openat2")]
pub fn pgsleuth_ebpf(_ctx: ProbeContext) -> u32 {
    0
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
