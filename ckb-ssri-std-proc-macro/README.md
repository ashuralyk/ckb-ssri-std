# ckb-ssri-std-proc-macro

Procedural macro for SSRI method dispatch. [`ckb-ssri-std`](../ckb-ssri-std/README.md) re-exports it, which is the dependency a contract should use.

Depend on this crate directly only when the macro is needed without the syscall library:

```toml
[dependencies]
ckb-ssri-std-proc-macro = "0.0.1"
```

The library is `proc-macro = true`. Its only public item is `ssri_methods`.

## `ssri_methods!`

```rust
ssri_methods!(
    argv: ckb_std::env::argv(),
    invalid_method: Error::SSRIMethodsNotFound,
    invalid_args: Error::SSRIMethodsArgsInvalid,
    "UDT.name" => Ok(Cow::Borrowed(b"MyToken")),
)
```

The three labels are parsed and discarded; the expressions are what matter. A comma is required after `invalid_args` and after every method arm. Each method name is a string literal.

The expansion is a block of type `Result<Cow<'static, [u8]>, Error>`. `Error` is resolved in the caller. `ckb_std::high_level::decode_hex` is called on `argv` elements, so that error must convert into `Error`, and each arm's expression must be the same `Result`.

`argv[0]` selects the method. It is hex bytes whose decoded form is exactly 8 bytes: the first 8 bytes of `blake2b-256(method_name)`, interpreted as a little-endian `u64`. A wrong length becomes `invalid_method`. An unknown id becomes `invalid_method` as well.

## Reserved methods

These three names are always present, ahead of the methods you list. Defining the same name again produces a duplicate match arm.

| Name | `argv` | Result |
| --- | --- | --- |
| `SSRI.version` | `[0]` only | a single byte `0` |
| `SSRI.get_methods` | `[1]` start index, `[2]` count | length-prefixed method ids from that index; count `0` returns every remaining id |
| `SSRI.has_methods` | `[1]` length-prefixed list of method ids | length-prefixed `0`/`1` bytes, one per requested id |

`SSRI.get_methods` reads `[1]` and `[2]` as hex-encoded 8-byte little-endian integers. A short or non-hex value becomes `invalid_args`. The returned bytes are a little-endian `u32` count, then that many little-endian `u64` ids. The ids are `SSRI.version`, `SSRI.get_methods`, `SSRI.has_methods`, then each custom method in source order.

`SSRI.has_methods` skips the first 4 bytes of the decoded `[1]` payload and compares each following 8-byte chunk with the known ids. `1` means the id is one of the reserved or custom methods.
