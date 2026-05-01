# pgsleuth-ebpf-poc

Source-only repo for the pgsleuth eBPF feasibility POC.

**This repo does not build on the host.** All compilation happens
inside the rust-dev container defined in the sibling `pgsleuth` repo
under `infra/docker/`. The POC plan and verdict live there too:

- Plan:    `pgsleuth/docs/research/ebpf-poc.md`
- Verdict: `pgsleuth/docs/research/ebpf-feasibility.md`

## Layout

```
pgsleuth-ebpf-poc/
├── Cargo.toml                # workspace
├── .cargo/config.toml        # build aliases (run inside container)
├── pgsleuth-ebpf/            # kernel-side eBPF program (no_std)
├── pgsleuth-ebpf-common/     # wire types shared with userspace
├── pgsleuth-ebpf-loader/     # userspace loader binary
└── xtask/                    # build orchestrator
```

## Build / run

From `pgsleuth/infra/docker/`:

```bash
docker compose run --rm rust-dev cargo build-ebpf
docker compose run --rm rust-dev cargo build -p pgsleuth-ebpf-loader
docker compose run --rm ebpf-feasibility pgsleuth-ebpf-loader
```

Exact compose service names are pinned by the POC plan, Step 3.

## Status

Stubs only. No probes attached, no events emitted. Step 4 of the
plan is the first iteration that does real work.
