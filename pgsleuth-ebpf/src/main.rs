#![no_std]
#![no_main]

// Kernel-side stub. Step 4 of docs/research/ebpf-poc.md (in the
// pgsleuth repo) will replace this with a real kprobe attached to
// `pread64` or `do_sys_openat2`.

use aya_ebpf::{macros::kprobe, programs::ProbeContext};

#[kprobe]
pub fn pgsleuth_ebpf(ctx: ProbeContext) -> u32 {
    match try_pgsleuth_ebpf(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_pgsleuth_ebpf(_ctx: ProbeContext) -> Result<u32, u32> {
    Ok(0)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
