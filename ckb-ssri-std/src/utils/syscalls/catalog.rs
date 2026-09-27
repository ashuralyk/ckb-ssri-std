//! SSRI syscall catalog.
//!
//! A syscall belongs here only when it has both a native-injection function and
//! an on-chain function with the same signature. To add one:
//!
//! 1. Add the same function to [`crate::utils::native`] and [`crate::utils::on_chain`].
//! 2. Add a [`Syscall`] row below.
//! 3. Call it from `high_level` with `syscall_branch!`.

/// System call number for finding an OutPoint by type script.
pub const SYS_FIND_OUT_POINT_BY_TYPE: u64 = 2277;
/// System call number for finding a cell by OutPoint.
pub const SYS_FIND_CELL_BY_OUT_POINT: u64 = 2287;
/// System call number for finding cell data by OutPoint.
pub const SYS_FIND_CELL_DATA_BY_OUT_POINT: u64 = 2297;
