# ckb-ssri-std

SDK for [Script-Sourced Rich Information (SSRI)](https://talk.nervos.org/t/en-cn-script-sourced-rich-information-script/8256) on Nervos CKB. SSRI binds a script's query methods to the script itself, so a contract can validate a transaction on-chain and answer the same questions off-chain.

This repository is a Cargo workspace with two crates:

| Crate | Path | Role |
| --- | --- | --- |
| [`ckb-ssri-std`](ckb-ssri-std/README.md) | `ckb-ssri-std` | `no_std` library contracts depend on: syscall helpers, indexer molecule types, and the `ssri_methods` re-export |
| [`ckb-ssri-std-proc-macro`](ckb-ssri-std-proc-macro/README.md) | `ckb-ssri-std-proc-macro` | Procedural macro that dispatches SSRI method names. Pulled in by `ckb-ssri-std` |

Both crates are version `0.0.1` and licensed MIT.

## Build and test

From this directory:

```sh
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

`ckb-ssri-std` is `no_std` and links `ckb-std`. Host tests run on the native target. RISC-V `ecall` assembly is compiled only for `target_arch = "riscv64"`; other targets use a stub.

## Related

- [SSRI introduction (EN/CN)](https://talk.nervos.org/t/en-cn-script-sourced-rich-information-script/8256)
- [`pausable-udt`](https://github.com/ckb-devrel/pausable-udt), a production contract that uses SSRI
- [`ssri-server`](https://github.com/ckb-devrel/ssri-server), a server that calls SSRI methods
- [CCC SSRI demo](https://ccc-git-udtssridemo-aaaaaaaalive24.vercel.app/?_vercel_share=zQkvWcsB2U9HRbpRFtF9w3xQT9msZDWb)
