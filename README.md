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
├── pgsleuth-pg-ext/          # Postgres extension (pgrx) — loader handshake
└── xtask/                    # build orchestrator
```

See `pgsleuth-pg-ext/README.md` for the extension's build and install
flow — that crate has its own toolchain story (`cargo-pgrx` +
`postgresql-server-dev-17`) and is Docker-only.

## Where things build

| Crate                    | Builds on host (macOS)? | Builds in Docker (Linux)? |
| ------------------------ | ----------------------- | ------------------------- |
| `pgsleuth-ebpf-common`   | yes                     | yes                       |
| `pgsleuth-ebpf-loader`   | yes (cargo check/build) | yes (runtime target)      |
| `xtask`                  | yes                     | yes                       |
| `pgsleuth-ebpf` (kernel) | **no** (nightly + bpf target) | **yes**             |
| `pgsleuth-pg-ext`        | **no** (pgrx + PG headers)    | **yes**             |

The workspace `default-members` excludes both `pgsleuth-ebpf` and
`pgsleuth-pg-ext`, so plain `cargo build` on the host stays on the
userspace crates only.

## Local dev (macOS host)

One-time setup — install rustup + stable Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile default
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
- `ebpf-feasibility` starts Postgres and waits for it to be ready
- Shares artifacts via `./ebpf-target` host directory mount

**Monitor the process:**
```bash
docker compose logs -f
```

### Step 5+ Workflow: Postgres + eBPF

**Start Postgres container:**
```bash
cd ../pgsleuth/infra/docker
docker compose up ebpf-feasibility
```

This runs the modular setup:
- `setup-postgres.sh` - Starts Postgres and waits for readiness
- Container stays running for manual eBPF loading

**Load eBPF program (after Postgres is ready):**
```bash
docker exec pgsleuth-ebpf-feasibility /workspace/load-ebpf.sh
```

**Connect to Postgres:**
```bash
docker exec -it pgsleuth-ebpf-feasibility psql -U postgres -h localhost
```

**Manual development workflow:**
```bash
# Get shell in running container
docker exec -it pgsleuth-ebpf-feasibility bash

# Inside container:
ps -ef | grep postgres          # Check Postgres processes
/workspace/load-ebpf.sh         # Load eBPF program
psql -U postgres -h localhost    # Connect to Postgres
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

✅ **Step 4 complete** — eBPF kprobe attached to `do_sys_openat2`, built in
`rust-dev`, loaded and verified in `ebpf-feasibility` (program ID 111, name
`pgsleuth_ebpf`). Container infrastructure and build script functional.

Next: Step 5 — swap the runtime container's base to `postgres:17-bookworm`
and trace `pread64` from a real backend. See the plan doc for the full
checkpoint list.

## Capabilities

The runtime container needs Linux capabilities to load and attach BPF
programs. Two practical options:

- **Easy / least surgery:** grant `CAP_SYS_ADMIN` alone. It implies the
  others and is the path of least resistance for local feasibility work.
  This is what makes ad-hoc `docker run` snippets shortest.
- **Least privilege (kernel ≥ 5.8):** grant `CAP_BPF` + `CAP_PERFMON`.
  More principled, but in practice we've also needed `CAP_SYS_ADMIN` for
  some operations (e.g. mounting bpffs, certain map types) and may need
  `CAP_NET_ADMIN` once we touch tracing helpers / network hooks.

The compose file (`pgsleuth/infra/docker/docker-compose.yml`) currently
grants **all of `BPF`, `PERFMON`, `NET_ADMIN`, `SYS_ADMIN`** — pragmatic
for a feasibility POC. Tightening this is a Phase 5 concern, not now.
