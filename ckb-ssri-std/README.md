# ckb-ssri-std

`no_std` library for SSRI-compliant CKB scripts. It selects a native or on-chain syscall backend from the invocation environment, parses indexer molecule values, and re-exports [`ssri_methods`](../ckb-ssri-std-proc-macro/README.md) for method dispatch.

```toml
[dependencies]
ckb-ssri-std = "0.0.1"
```

The `on-chain` feature compiles `syscalls::on_chain`. It is off by default. Enable it when a script must answer these syscalls during transaction verification:

```toml
ckb-ssri-std = { version = "0.0.1", features = ["on-chain"] }
```

The crate is `#![no_std]` and uses `alloc`. `ckb-ssri-std-proc-macro` is a dependency, so `ssri_methods!` is available from the crate root without a second dependency.

## Layout

| Path | What it is |
| --- | --- |
| crate root | `syscall_branch!`, `should_fallback`, `SSRIError`, and `ssri_methods` |
| `high_level` | Syscalls that return parsed molecule or `ckb_types` values |
| `indexer` | `serde_molecule` types for `get_cells` and `get_live_cell` |
| `syscalls` | `native` backend, optional `on_chain` backend (`on-chain` feature), syscall numbers in `catalog`, and the `raw` `ecall` |
| `prelude` | Length-prefixed codecs for `u64` and `[u8; 32]` vectors |

Call `high_level` from a script. `syscall_branch!` chooses `syscalls::native` or, when the `on-chain` feature is enabled, `syscalls::on_chain`. A syscall is part of SSRI only when both backends exist and share a signature.

## Backend selection

`should_fallback` reads the current environment:

| Condition | Result |
| --- | --- |
| `argv` is empty | `Ok(true)`, use `on_chain` when the `on-chain` feature is enabled |
| `argv` is present and `vm_version` is `u64::MAX` | `Ok(false)`, use `native` |
| `argv` is present and `vm_version` is anything else | `Err(SSRIError::InvalidVmVersion)` |

Empty `argv` is transaction verification: the on-chain backend answers from cell deps, header deps, and inputs. A present `argv` with the host stub (`vm_version == u64::MAX`) is native injection, where the host serves the same calls. `syscall_branch!` turns `InvalidVmVersion` into `SysError::Unknown(u64::MAX)`. The same error is returned for `Ok(true)` when the `on-chain` feature is disabled.

```rust
use ckb_ssri_std::syscall_branch;

let len = syscall_branch!(network(&mut buf))?;
```

With `on-chain` enabled, the macro calls `on_chain::network` or `native::network` with the same arguments. Without that feature it calls `native::network` only when `should_fallback` is `Ok(false)`.

## High-level calls

Each function loads the full syscall buffer, checks the encoding, and returns a typed value. Errors are `ckb_std::error::SysError`.

| Function | Returns |
| --- | --- |
| `find_out_point_by_type(type_script)` | `OutPoint` of the first cell with that type script |
| `find_cell_by_out_point(out_point)` | `CellOutput` |
| `find_cell_data_by_out_point(out_point)` | cell data bytes |
| `network()` | node info bytes (`local_node_info`) |
| `get_live_cell(out_point, with_data)` | `indexer::LiveCell` |
| `get_header(block_hash)` | `Header` |
| `get_header_by_number(block_number)` | `Header` |
| `get_block_hash(block_number)` | `Byte32` |
| `get_transaction_block_hash(tx_hash)` | block hash from `get_transaction`'s `tx_status.block_hash` |
| `get_cells(search_key, order, limit, after)` | `indexer::Pagination` |

`get_live_cell` and `get_cells` decode molecule bytes with `serde_molecule`. `SearchKey` follows ckb-indexer's search-key field order. `Order::Asc` is `0` and `Order::Desc` is `1`. `after` is the pagination cursor.

Syscall numbers live in `syscalls::catalog` (`2277` through `2367`).

## Method dispatch

`ssri_methods!` is re-exported from `ckb-ssri-std-proc-macro`. The block's type is `Result<Cow<'static, [u8]>, Error>`, and `Error` must already be in scope. `decode_hex` failures must convert into that `Error`.

```rust
use alloc::borrow::Cow;
use ckb_ssri_std::ssri_methods;

let res = ssri_methods!(
    argv: ckb_std::env::argv(),
    invalid_method: Error::SSRIMethodsNotFound,
    invalid_args: Error::SSRIMethodsArgsInvalid,
    "UDT.name" => Ok(Cow::Borrowed(b"MyToken")),
);
```

`argv[0]` is the hex encoding of the method id: the first 8 bytes of `blake2b-256(name)`, read as a little-endian `u64`. These names are reserved and always dispatched by the macro:

| Name | Behavior |
| --- | --- |
| `SSRI.version` | one byte, `0` |
| `SSRI.get_methods` | a page of method ids; `argv[1]` is the start index and `argv[2]` is the count (`0` means the rest) |
| `SSRI.has_methods` | one `0` or `1` per id in `argv[1]` |

Each custom arm is `"name" => expr,` and `expr` has the same `Result` type as the block. A trailing comma is required after `invalid_args` and after every arm. See the [proc-macro README](../ckb-ssri-std-proc-macro/README.md) for the argv layout.

## `SSRIError`

`SSRIError` is `#[repr(i8)]` so a script `Error` can include it:

- `SSRIMethodsNotFound`
- `SSRIMethodsArgsInvalid`
- `SSRIMethodsNotImplemented`
- `SSRIMethodRequireHigherLevel`
- `InvalidVmVersion`

## Prelude codecs

`prelude` writes a little-endian `u32` length, then the payload:

- `encode_u64_vector` / `decode_u64_vector` for `&[u64]`
- `encode_u8_32_vector` / `decode_u8_32_vector` for `&[[u8; 32]]`

## Testing a contract

`ckb_testtools` covers on-chain verification. Off-chain queries and transaction assembly need `ckb_ssri_cli` against the latest deployment of the same contract.

This crate's own tests:

```sh
cargo test -p ckb-ssri-std --all-targets --all-features
```
