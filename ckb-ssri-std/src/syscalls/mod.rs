//! SSRI syscalls.
//!
//! [`crate::syscall_branch`] chooses [`native`] or [`on_chain`] for the
//! same function name. [`catalog`] is the list of syscalls that have both.

// re-export to maintain compatible with old versions
pub use ckb_std::error::SysError;

pub mod catalog;
pub mod native;
pub mod on_chain;
pub mod raw;
