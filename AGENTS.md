## Learned User Preferences

- Prefer `crate::` absolute imports over `super::`; keep module paths shallow (at most two `::`, preferably one) and place shared types in the parent module rather than nested bag modules.
- When SSRI syscall behavior is underspecified, ask before guessing.
- Keep existing syscall function signatures unchanged unless explicitly asked to change them.
- Choose native vs on-chain backends at runtime (`should_fallback` / a branch helper macro), not via Cargo feature `cfg`.
- Add tests covering each syscall.
- Keep syscall-framework refactors scoped to the `ckb-ssri-std` project.
- Prefer inlining single-expression helpers so each caller holds the expression directly.
- For hex-encoded indexer/JSON fields, use `ckb_std`'s `hex_string` and do not add a `0x` prefix.
- Keep `cargo clippy --all-targets --all-features` free of warnings and errors.

## Learned Workspace Facts

- SSRI syscalls must support both native injection and on-chain adaptation with matching semantics; one-sided APIs are not qualified for SSRI.
- Syscall layout is catalog + `raw` + `native` + `on_chain`, with `high_level` selecting the backend from the invocation environment.
- Native backends use host/indexer-style injection; on-chain backends resolve the same meaning from the current transaction (cell deps, header deps, and related fields).
- Host builds gate RISC-V `ecall` assembly behind `target_arch = "riscv64"` and stub otherwise.
- The catalog includes CKB/ckb-indexer-aligned SSRI syscalls such as `network`, `get_live_cell`, `get_header`, `get_header_by_number`, `get_block_hash`, `get_transaction_block_hash`, and `get_cells`.
- Indexer JSON response types (cell/pagination shapes) live in `utils/indexer.rs` without a `Json` prefix; hex script naming avoids clashing with molecule `Script` (e.g. `HexScript`).
- `get_cells` search keys use a `serde_molecule` type matching ckb-indexer's `SearchKey` layout.
- Syscall return buffers for indexer-aligned methods (`get_cells`, `get_live_cell`) target ckb-indexer RPC JSON bytes; molecule `LiveCell` is a post-parse client type, not the syscall payload.
