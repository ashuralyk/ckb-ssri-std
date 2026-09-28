use ckb_std::debug;
use syscalls::vm_version;

use crate::SSRIError;

pub mod high_level;
pub mod indexer;
pub mod syscalls;

pub use indexer::{
    CellOutput, IndexerCell, LiveCell, Order, OutPoint, Pagination, Script, ScriptType, SearchKey,
    SearchKeyFilter, SearchMode,
};

pub(crate) use syscalls::{catalog, native, on_chain, raw, SysError};

/// Choose the on-chain or native-injection syscall of the same name.
///
/// `should_fallback` is the environment check: `Ok(true)` calls `on_chain::$name`,
/// `Ok(false)` calls `native::$name`. `InvalidVmVersion` becomes `SysError::Unknown(u64::MAX)`.
macro_rules! syscall_branch {
    ($name:ident($($arg:expr),* $(,)?)) => {{
        use $crate::utils;
        use utils::{native, on_chain, should_fallback, SysError};
        match should_fallback() {
            Ok(true) => on_chain::$name($($arg),*),
            Ok(false) => native::$name($($arg),*),
            Err(_) => Err(SysError::Unknown(u64::MAX)),
        }
    }};
}
pub(crate) use syscall_branch;

pub fn should_fallback() -> Result<bool, SSRIError> {
    if ckb_std::env::argv().is_empty() {
        debug!("Should fallback!");
        return Ok(true);
    } else {
        if vm_version() != u64::MAX {
            return Err(SSRIError::InvalidVmVersion);
        } else {
            debug!("Should not fallback!");
            return Ok(false);
        }
    }
}
