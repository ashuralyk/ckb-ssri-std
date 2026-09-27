//! SSRI syscall catalog.
//!
//! A syscall belongs here only when it has both a native-injection function and
//! an on-chain function with the same signature. To add one:
//!
//! 1. Add the same function to [`crate::utils::native`] and [`crate::utils::on_chain`].
//! 2. Add its number here.
//! 3. Call it from `high_level` with `syscall_branch!`.

/// System call number for finding an OutPoint by type script.
pub const SYS_FIND_OUT_POINT_BY_TYPE: u64 = 2277;
/// System call number for finding a cell by OutPoint.
pub const SYS_FIND_CELL_BY_OUT_POINT: u64 = 2287;
/// System call number for finding cell data by OutPoint.
pub const SYS_FIND_CELL_DATA_BY_OUT_POINT: u64 = 2297;
/// CKB net RPC `local_node_info`. Native: host ecall. On-chain: no transaction field yet.
pub const SYS_NETWORK: u64 = 2307;
/// CKB RPC `get_live_cell`. Native: host ecall. On-chain: the same out point in the transaction.
pub const SYS_GET_LIVE_CELL: u64 = 2317;
/// CKB RPC `get_header`. Native: host ecall. On-chain: the matching header dep.
pub const SYS_GET_HEADER: u64 = 2327;
/// CKB RPC `get_header_by_number`. Native: host ecall. On-chain: the header dep with that number.
pub const SYS_GET_HEADER_BY_NUMBER: u64 = 2337;
/// CKB RPC `get_block_hash`. Native: host ecall. On-chain: the hash of the header dep with that number.
pub const SYS_GET_BLOCK_HASH: u64 = 2347;
/// Block hash of a committed transaction (`get_transaction` `tx_status.block_hash`).
/// Native: host ecall. On-chain: the block that committed that transaction.
pub const SYS_GET_TRANSACTION_BLOCK_HASH: u64 = 2357;
/// ckb-indexer RPC `get_cells`. Native: host ecall. On-chain: cells in the transaction that match the search.
pub const SYS_GET_CELLS: u64 = 2367;
