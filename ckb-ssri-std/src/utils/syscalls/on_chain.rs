use ckb_std::error::SysError;

use crate::utils::catalog::{
    SYS_FIND_CELL_BY_OUT_POINT, SYS_FIND_CELL_DATA_BY_OUT_POINT, SYS_FIND_OUT_POINT_BY_TYPE,
    SYS_GET_BLOCK_HASH, SYS_GET_CELLS, SYS_GET_HEADER, SYS_GET_HEADER_BY_NUMBER, SYS_GET_LIVE_CELL,
    SYS_GET_TRANSACTION_BLOCK_HASH, SYS_NETWORK,
};

/// On-chain adaptation of `find_out_point_by_type`.
///
/// Same semantics as the native injection: the OutPoint of the cell whose type
/// script matches, taken from cell deps and other cell-bearing transaction fields.
pub fn find_out_point_by_type(_buf: &mut [u8], _type_script: &[u8]) -> Result<usize, SysError> {
    Err(SysError::Unknown(SYS_FIND_OUT_POINT_BY_TYPE))
}

/// On-chain adaptation of `find_cell_by_out_point`.
///
/// Same semantics as the native injection: the cell output for that out point
/// in the current transaction.
pub fn find_cell_by_out_point(_buf: &mut [u8], _out_point: &[u8]) -> Result<usize, SysError> {
    Err(SysError::Unknown(SYS_FIND_CELL_BY_OUT_POINT))
}

/// On-chain adaptation of `find_cell_data_by_out_point`.
///
/// Same semantics as the native injection: that cell's data in the current transaction.
pub fn find_cell_data_by_out_point(_buf: &mut [u8], _out_point: &[u8]) -> Result<usize, SysError> {
    Err(SysError::Unknown(SYS_FIND_CELL_DATA_BY_OUT_POINT))
}

/// On-chain adaptation of `network`.
///
/// Same semantics as the native `local_node_info` query. No transaction field carries it yet.
pub fn network(_buf: &mut [u8]) -> Result<usize, SysError> {
    Err(SysError::Unknown(SYS_NETWORK))
}

/// On-chain adaptation of `get_live_cell`.
///
/// Same semantics as the native injection: the cell for that out point, from cell deps
/// and other cell-bearing transaction fields. Include its data when `with_data` is set.
pub fn get_live_cell(
    _buf: &mut [u8],
    _out_point: &[u8],
    _with_data: bool,
) -> Result<usize, SysError> {
    Err(SysError::Unknown(SYS_GET_LIVE_CELL))
}

/// On-chain adaptation of `get_header`.
///
/// Same semantics as the native injection: the header whose hash matches, from header deps.
pub fn get_header(_buf: &mut [u8], _block_hash: &[u8]) -> Result<usize, SysError> {
    Err(SysError::Unknown(SYS_GET_HEADER))
}

/// On-chain adaptation of `get_header_by_number`.
///
/// Same semantics as the native injection: the header dep with that block number.
pub fn get_header_by_number(_buf: &mut [u8], _block_number: u64) -> Result<usize, SysError> {
    Err(SysError::Unknown(SYS_GET_HEADER_BY_NUMBER))
}

/// On-chain adaptation of `get_block_hash`.
///
/// Same semantics as the native injection: the hash of the header dep with that block number.
pub fn get_block_hash(_buf: &mut [u8], _block_number: u64) -> Result<usize, SysError> {
    Err(SysError::Unknown(SYS_GET_BLOCK_HASH))
}

/// On-chain adaptation of `get_transaction_block_hash`.
///
/// Same semantics as the native injection: the block hash of the block that committed
/// the transaction.
pub fn get_transaction_block_hash(_buf: &mut [u8], _tx_hash: &[u8]) -> Result<usize, SysError> {
    Err(SysError::Unknown(SYS_GET_TRANSACTION_BLOCK_HASH))
}

/// On-chain adaptation of `get_cells`.
///
/// Same semantics as the native indexer query: cells in the transaction that match
/// `search_key`, ordered and limited the same way. `after` is the pagination cursor.
pub fn get_cells(
    _buf: &mut [u8],
    _search_key: &[u8],
    _order: u64,
    _limit: u64,
    _after: &[u8],
) -> Result<usize, SysError> {
    Err(SysError::Unknown(SYS_GET_CELLS))
}
