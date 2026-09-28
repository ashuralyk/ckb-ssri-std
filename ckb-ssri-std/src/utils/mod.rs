use ckb_std::debug;

use crate::SSRIError;

pub mod high_level;
pub mod indexer;
pub mod syscalls;

/// Choose the on-chain or native-injection syscall of the same name.
///
/// `should_fallback` is the environment check: `Ok(true)` calls `on_chain::$name`,
/// `Ok(false)` calls `native::$name`. `InvalidVmVersion` becomes `SysError::Unknown(u64::MAX)`.
#[macro_export]
macro_rules! syscall_branch {
    ($name:ident($($arg:expr),* $(,)?)) => {{
        use $crate::utils::{syscalls::{native, on_chain}, should_fallback};
        match should_fallback() {
            Ok(true) => on_chain::$name($($arg),*),
            Ok(false) => native::$name($($arg),*),
            Err(_) => Err(ckb_std::error::SysError::Unknown(u64::MAX)),
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
