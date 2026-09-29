#![no_std]
//! # CKB SSRI std
//!
//! Utils for implementing SSRI-compliant smart contracts on the Nervos CKB blockchain.
//!
//! ## Overview
//!
//! - SSRI stands for `Script Sourced Rich Information`; it is a protocol for strong bindings of relevant information and conventions to the Script itself on CKB. For more information, please read [[EN/CN] Script-Sourced Rich Information - 来源于 Script 的富信息](https://talk.nervos.org/t/en-cn-script-sourced-rich-information-script/8256)>.
//! - For writing CKB Scripts (or "Smart Contracts"), by selectively implementing methods of public module traits (e.g. `UDT`, `UDTExtended`, `UDTPausable`) in combinations, devs would be able to quickly design and organize functionalities that either validate transactions or provide rich information as well as assembling transactions off-chain.
//! - For dApps or other infrastructures that interact with CKB Scripts, you no longer need to retrieve and parse data or assemble transactions by yourself repetitively as they are all provided by SSRI.

//!
//! ## Features
//!
//! - **Utility Functions**: Helper functions for SSRI-VM syscalls and data handling
//! - **Procedural Macros**: Simplify contract development with automatic SSRI method generation
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! ckb-ssri-std = "0.1.0"
//! ```
//!
//! ## Example
//! [`pausable-udt`](https://github.com/ckb-devrel/pausable-udt) is a real production level contract (instead of a pseudo-project) that exemplifies the usage of SSRI.

pub mod high_level;
pub mod indexer;
pub mod macros;
pub mod prelude;
pub mod syscalls;

// Re-export proc macros at crate root for convenience
pub use macros::*;

extern crate alloc;

use ckb_std::debug;

/// Choose the on-chain or native-injection syscall of the same name.
///
/// `should_fallback` is the environment check. With the `on-chain` feature,
/// `Ok(true)` calls `on_chain::$name` and `Ok(false)` calls `native::$name`.
/// Without that feature, `Ok(true)` returns `SysError::Unknown(u64::MAX)`.
/// `InvalidVmVersion` becomes the same error.
#[cfg(feature = "on-chain")]
#[macro_export]
macro_rules! syscall_branch {
    ($name:ident($($arg:expr),* $(,)?)) => {{
        use $crate::{syscalls::{native, on_chain}, should_fallback};
        match should_fallback() {
            Ok(true) => on_chain::$name($($arg),*),
            Ok(false) => native::$name($($arg),*),
            Err(_) => Err(ckb_std::error::SysError::Unknown(u64::MAX)),
        }
    }};
}

/// Choose the native-injection syscall of the given name.
///
/// The `on-chain` feature is disabled, so `should_fallback` returning `Ok(true)`
/// or `InvalidVmVersion` becomes `SysError::Unknown(u64::MAX)`. `Ok(false)` calls
/// `native::$name`.
#[cfg(not(feature = "on-chain"))]
#[macro_export]
macro_rules! syscall_branch {
    ($name:ident($($arg:expr),* $(,)?)) => {{
        use $crate::{should_fallback, syscalls::native};
        match should_fallback() {
            Ok(false) => native::$name($($arg),*),
            _ => Err(ckb_std::error::SysError::Unknown(u64::MAX)),
        }
    }};
}

pub fn should_fallback() -> Result<bool, SSRIError> {
    if ckb_std::env::argv().is_empty() {
        debug!("Should fallback!");
        Ok(true)
    } else if syscalls::raw::vm_version() != u64::MAX {
        Err(SSRIError::InvalidVmVersion)
    } else {
        debug!("Should not fallback!");
        Ok(false)
    }
}

#[repr(i8)]
#[derive(Debug)]
/// Represents possible errors that can occur during SSRI method execution
/// Should be derivable to `Error` for the actual script to use.
pub enum SSRIError {
    /// The requested SSRI method was not found in the contract
    SSRIMethodsNotFound,
    /// The arguments provided to the SSRI method were invalid
    SSRIMethodsArgsInvalid,
    /// The requested SSRI method is not implemented
    SSRIMethodsNotImplemented,
    /// The method requires a higher execution environment level
    SSRIMethodRequireHigherLevel,
    /// The CKB VM version is not compatible with this implementation
    InvalidVmVersion,
}
