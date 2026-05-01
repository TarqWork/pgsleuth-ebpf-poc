// xtask stub. Will gain `build-ebpf` and `run` subcommands as the
// POC progresses.

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "xtask", version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Build the kernel-side eBPF crate for `bpfel-unknown-none`.
    BuildEbpf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::BuildEbpf => {
            println!("xtask build-ebpf: not yet implemented");
            Ok(())
        }
    }
}
