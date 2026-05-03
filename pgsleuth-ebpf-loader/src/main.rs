// Userspace loader for Step 4 of docs/research/ebpf-poc.md
// Minimal loader - just loads and attaches the kprobe, no perf events.

use anyhow::{Context, Result};
use aya::{
    programs::KProbe,
    Ebpf,
};
use aya_log::EbpfLogger;
use clap::Parser;
use log::{info, warn};
use tokio::signal;

#[derive(Debug, Parser)]
#[command(name = "pgsleuth-ebpf-loader", version, about)]
struct Args {
    /// Path to the compiled BPF object emitted by the rust-dev container.
    #[arg(long, default_value = "/build/pgsleuth-ebpf")]
    bpf_object: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!("Loading BPF object from: {}", args.bpf_object);

    // Load the BPF program
    let mut bpf = Ebpf::load_file(&args.bpf_object)
        .context("Failed to load BPF object")?;

    // Initialize the eBPF logger (optional, may not be present in minimal program)
    if let Err(e) = EbpfLogger::init(&mut bpf) {
        warn!("Failed to initialize eBPF logger: {}", e);
    }

    // Load and attach the kprobe
    let program: &mut KProbe = bpf.program_mut("pgsleuth_ebpf")
        .unwrap()
        .try_into()
        .context("Failed to get program as KProbe")?;

    program.load()
        .context("Failed to load BPF program")?;

    program.attach("do_sys_openat2", 0)
        .context("Failed to attach kprobe to do_sys_openat2")?;

    info!("Successfully attached kprobe to do_sys_openat2");
    info!("eBPF program is running. Press Ctrl+C to stop.");

    // Wait for Ctrl+C
    signal::ctrl_c().await.context("Failed to wait for Ctrl+C")?;
    info!("Received Ctrl+C, shutting down...");

    Ok(())
}
