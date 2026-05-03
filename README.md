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
- `rust-dev` runs the build script (`bash /workspace/build.sh ebpf`)
- `ebpf-feasibility` attempts to load and run the eBPF program
- Shares artifacts via `./ebpf-target` host directory mount

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

### Using the Build Script

The `rust-dev` container includes a build script with multiple commands:

```bash
cd ../pgsleuth/infra/docker

# Build eBPF only (default)
docker compose run rust-dev bash /workspace/build.sh ebpf

# Build everything (eBPF + loader + common)
docker compose run rust-dev bash /workspace/build.sh all

# Build specific components
docker compose run rust-dev bash /workspace/build.sh loader
docker compose run rust-dev bash /workspace/build.sh common

# Clean artifacts
docker compose run rust-dev bash /workspace/build.sh clean
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

**Direct cargo via Docker compose (current workflow)**
```bash
# From pgsleuth/infra/docker directory:
docker compose run rust-dev bash /workspace/build.sh ebpf

# Or run cargo directly inside the container:
docker compose exec rust-dev bash
cd /workspace/source/pgsleuth-ebpf-poc
cargo build -Zbuild-std --target bpfel-unknown-none --release -p pgsleuth-ebpf
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

🔄 **Step 4 In Progress** - eBPF kprobe attached to `do_sys_openat2`
- eBPF program **builds successfully** in Docker (`rust-dev` container)
- Container infrastructure and build script functional
- **Caveat:** eBPF load/verify step in `ebpf-feasibility` container not yet fully tested end-to-end
- Ready for Step 5 (Postgres integration) once load is verified
