# pgsleuth-ebpf-poc

Source for the pgsleuth eBPF feasibility POC. Plan and verdict live
in the sibling `pgsleuth` repo:

- Plan:    `pgsleuth/docs/research/ebpf-poc.md`
- Verdict: `pgsleuth/docs/research/ebpf-feasibility.md`

## Layout

```
pgsleuth-ebpf-poc/
├── Cargo.toml                # workspace
├── .cargo/config.toml        # build aliases (Docker-only)
├── pgsleuth-ebpf/            # kernel-side eBPF program (no_std)
├── pgsleuth-ebpf-common/     # wire types shared with userspace
├── pgsleuth-ebpf-loader/     # userspace loader binary
└── xtask/                    # build orchestrator
```

## Where things build

| Crate                    | Builds on host (macOS)? | Builds in Docker (Linux)? |
| ------------------------ | ----------------------- | ------------------------- |
| `pgsleuth-ebpf-common`   | yes                     | yes                       |
| `pgsleuth-ebpf-loader`   | yes (cargo check/build) | yes (runtime target)      |
| `xtask`                  | yes                     | yes                       |
| `pgsleuth-ebpf` (kernel) | **no** (nightly + bpf target) | **yes**             |

The workspace `default-members` excludes `pgsleuth-ebpf`, so plain
`cargo build` on the host stays away from the kernel crate.

## Local dev (macOS host)

One-time setup — install rustup + stable Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs  sh -s -- -y --default-toolchain stable --profile default
source "$HOME/.cargo/env"
rustup component add rustfmt clippy
```

Day-to-day:

```bash
cargo check                                  # userspace crates
cargo build -p pgsleuth-ebpf-loader          # userspace loader
cargo fmt
cargo clippy
```

Do **not** install nightly or the `bpfel-unknown-none` target on the
host. The kernel crate is built in Docker.

## Building/running the eBPF side (Docker)

The `rust-dev` and `ebpf-feasibility` containers are defined in
`../pgsleuth/infra/docker/`.

### Automated Build & Load (Recommended)

**One-command build and load:**
```bash
cd ../pgsleuth/infra/docker
docker compose up --build
```

This automatically:
- Builds the eBPF program in `rust-dev` container
- Loads and runs the eBPF program in `ebpf-feasibility` container  
- Shares artifacts via `bpf-builds` volume

**Monitor the process:**
```bash
docker compose logs -f
```

### Individual Commands

**Build eBPF program only:**
```bash
cd ../pgsleuth/infra/docker
docker compose run rust-dev
```

**Load and run eBPF program only:**
```bash
cd ../pgsleuth/infra/docker  
docker compose run ebpf-feasibility
```

**From pgsleuth-ebpf-poc directory:**
```bash
# Build eBPF program
docker run --rm -v $(pwd):/workspace/source -v ebpf-builds:/workspace/target \
  pgsleuth/rust-dev cargo build -Zbuild-std --target bpfel-unknown-none --release -p pgsleuth-ebpf

# Load and run eBPF program  
docker run --rm --cap-add=BPF --cap-add=PERFMON --cap-add=SYS_ADMIN \
  -v ebpf-builds:/workspace/target -v /sys/fs/bpf:/sys/fs/bpf \
  pgsleuth/ebpf-feasibility sh -c "mount -t bpf bpf /sys/fs/bpf && bpftool prog load bpfel-unknown-none/release/pgsleuth-ebpf /sys/fs/bpf/pgsleuth-ebpf"
```

### Manual Development

**Get shell in build container:**
```bash
cd ../pgsleuth/infra/docker
docker compose run --rm rust-dev bash
```

**Get shell in runtime container:**
```bash
cd ../pgsleuth/infra/docker
docker compose run --rm ebpf-feasibility bash
```

Exact service names are pinned by the POC plan, Step 3.

## FAQ

### Why use xtask vs direct cargo commands?

**Short answer:** For the current simple eBPF build, xtask is optional overhead. Direct cargo works fine.

**Current situation (Step 4):**
```bash
# Direct cargo - simpler, works perfectly
docker run --rm -v $(pwd):/workspace -w /workspace pgsleuth/rust-dev \
  cargo build -Zbuild-std --target bpfel-unknown-none --release -p pgsleuth-ebpf

# xtask - more verbose, same result
docker run --rm -v $(pwd):/workspace -w /workspace pgsleuth/rust-dev \
  cargo run --bin xtask -- build-ebpf
```

**Why xtask was added:**
- **Future-proofing** - eBPF builds will get more complex (multiple steps, verification, artifact copying)
- **Standard practice** - xtask is the conventional Rust way for project automation
- **Better UX** - More discoverable commands, better error messages

**When xtask becomes valuable (future steps):**
```rust
// Multi-step processes like:
Cmd::BuildAll => {
    build_ebpf_program()?;      // Build kernel side
    build_userspace_loader()?;  // Build userspace side  
    copy_artifacts()?;         // Copy to shared volume
    generate_bpf_skeleton()?;  // For advanced eBPF features
    verify_bytecode()?;         // Safety checks
}
```

**Recommendation:** Use direct cargo for now. Switch to xtask when the build process becomes complex enough to justify the abstraction.

### Why not just install eBPF tooling on macOS?

**Cross-platform compatibility:** eBPF tooling (bpf-linker, BPF targets) is Linux-specific. macOS:
- Can't install `bpfel-unknown-none` target
- Can't run eBPF programs (kernel support missing)
- Has different toolchain ecosystem

**Docker approach provides:**
- Consistent Linux build environment
- All required eBPF tooling pre-installed
- Same environment that will actually run the eBPF programs

## Status

✅ **Step 4 Complete** - Working eBPF kprobe attached to `do_sys_openat2`
- eBPF program builds and loads successfully
- Container infrastructure functional
- Ready for Step 5 (Postgres integration)
