#![cfg_attr(not(feature = "user"), no_std)]

// Wire types shared between kernel-side eBPF and userspace loader.

use aya_ebpf_cty::c_long;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SyscallEvent {
    pub pid: u32,
    pub syscall_nr: c_long,
    pub filename_ptr: u64,
}
