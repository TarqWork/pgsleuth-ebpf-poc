// Userspace loader stub. Real attach/poll logic lands with Step 4
// of docs/research/ebpf-poc.md.

use anyhow::Result;
use clap::Parser;

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
    log::info!("pgsleuth-ebpf-loader stub; would load {}", args.bpf_object);
    Ok(())
}
