## Learned User Preferences

- Prefer `crate::` absolute imports over `super::`; keep module paths shallow (at most two `::`, preferably one) and place shared types in the parent module rather than nested bag modules.
- When SSRI syscall behavior is underspecified, ask before guessing.
- Keep existing syscall function signatures unchanged unless explicitly asked to change them.
- Choose native vs on-chain backends at runtime (`should_fallback` / a branch helper macro). The `on-chain` Cargo feature only includes or omits the on-chain backend; it is off by default.
- Add tests covering each syscall.
- Keep syscall-framework refactors scoped to the `ckb-ssri-std` project.
- Prefer inlining single-expression helpers so each caller holds the expression directly.
- Prefer `serde_molecule` (`to_vec` / `from_slice`) over hand-rolled molecule table assemble/disassemble helpers.
- Keep `cargo clippy --all-targets --all-features` free of warnings and errors.

## Learned Workspace Facts

- SSRI syscalls must support both native injection and on-chain adaptation with matching semantics; one-sided APIs are not qualified for SSRI.
- Syscall layout is catalog + `raw` + `native` + `on_chain`. `on_chain` compiles only with the `on-chain` feature (off by default). With that feature, `high_level` selects the backend from the invocation environment.
- Native backends use host/indexer-style injection; on-chain backends resolve the same meaning from the current transaction (cell deps, header deps, and related fields).
- Host builds gate RISC-V `ecall` assembly behind `target_arch = "riscv64"` and stub otherwise.
- The catalog includes CKB/ckb-indexer-aligned SSRI syscalls such as `network`, `get_live_cell`, `get_header`, `get_header_by_number`, `get_block_hash`, `get_transaction_block_hash`, and `get_cells`.
- Crate modules live at the root (`high_level`, `indexer`, `syscalls`); there is no `public_module_traits` or nested `utils` wrapper.
- Indexer/pagination types (`LiveCell`, `IndexerCell`, `Pagination`, `Order`, `SearchKey`) live in `indexer.rs` and use `serde_molecule` (packed `CellOutput`/`OutPoint`/bytes), not hex-string JSON shapes.
- `get_cells` takes an `Order` enum (`Asc` = 0, `Desc` = 1) and `last_cursor: &[u8]` (empty starts the first page); search keys use a `serde_molecule` type matching ckb-indexer's `SearchKey` layout.
- Syscall return buffers for `get_cells` / `get_live_cell` are `serde_molecule` bytes; `high_level` decodes them to `Pagination` / `LiveCell`.
