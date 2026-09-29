use ckb_std::error::SysError;

use crate::{
    indexer::{Order, SearchKey},
    syscalls::{catalog::*, raw::syscall_load},
};
use alloc::vec::Vec;

/// Find an OutPoint by searching for a specific type script.
///
/// Native injection: the host serves this through `ecall` from ckb-indexer.
/// The OutPoint data is written to the provided buffer.
///
/// # Arguments
///
/// * `buf` - A mutable buffer to receive the OutPoint data
/// * `type_script` - The serialized type script to search for
///
/// # Returns
///
/// * `Ok(usize)` - The actual length of the OutPoint data written to the buffer
/// * `Err(SysError)` - A system error if the operation fails
///
/// # Errors
///
/// Returns `SysError::LengthNotEnough` if the buffer is too small to hold the data
/// Returns `SysError::IndexOutOfBound` if the type script is invalid
/// Returns `SysError::ItemMissing` if no matching cell is found
pub fn find_out_point_by_type(buf: &mut [u8], type_script: &[u8]) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        type_script.as_ptr() as usize,
        type_script.len() as u64,
        0,
        0,
        0,
        SYS_FIND_OUT_POINT_BY_TYPE,
    )
}

/// Find a cell by its OutPoint.
///
/// Native injection: the host serves this through `ecall`.
/// The cell data is written to the provided buffer.
///
/// # Arguments
///
/// * `buf` - A mutable buffer to receive the cell data
/// * `out_point` - The serialized OutPoint identifying the cell to find
///
/// # Returns
///
/// * `Ok(usize)` - The actual length of the cell data written to the buffer
/// * `Err(SysError)` - A system error if the operation fails
///
/// # Errors
///
/// Returns `SysError::LengthNotEnough` if the buffer is too small to hold the data
/// Returns `SysError::IndexOutOfBound` if the OutPoint is invalid
/// Returns `SysError::ItemMissing` if the cell cannot be found
pub fn find_cell_by_out_point(buf: &mut [u8], out_point: &[u8]) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        out_point.as_ptr() as usize,
        0,
        0,
        0,
        0,
        SYS_FIND_CELL_BY_OUT_POINT,
    )
}

/// Find cell data by OutPoint.
///
/// Native injection: the host serves this through `ecall`.
/// The cell's data is written to the provided buffer.
///
/// # Arguments
///
/// * `buf` - A mutable buffer to receive the cell's data
/// * `out_point` - The serialized OutPoint identifying the cell whose data to retrieve
///
/// # Returns
///
/// * `Ok(usize)` - The actual length of the cell data written to the buffer
/// * `Err(SysError)` - A system error if the operation fails
///
/// # Errors
///
/// Returns `SysError::LengthNotEnough` if the buffer is too small to hold the data
/// Returns `SysError::IndexOutOfBound` if the OutPoint is invalid
/// Returns `SysError::ItemMissing` if the cell cannot be found
pub fn find_cell_data_by_out_point(buf: &mut [u8], out_point: &[u8]) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        out_point.as_ptr() as usize,
        0,
        0,
        0,
        0,
        SYS_FIND_CELL_DATA_BY_OUT_POINT,
    )
}

/// CKB net RPC `local_node_info`.
///
/// Native injection: the host writes the node info into `buf`.
/// No query bytes. `a2` through `a6` are zero.
pub fn network(buf: &mut [u8]) -> Result<usize, SysError> {
    syscall_load(buf.as_mut_ptr(), buf.len(), 0, 0, 0, 0, 0, SYS_NETWORK)
}

/// CKB RPC `get_live_cell`.
///
/// Native injection: `a2` is the OutPoint, `a3` is `1` when cell data is requested.
pub fn get_live_cell(buf: &mut [u8], out_point: &[u8], with_data: bool) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        out_point.as_ptr() as usize,
        u64::from(with_data),
        0,
        0,
        0,
        SYS_GET_LIVE_CELL,
    )
}

/// CKB RPC `get_header`.
///
/// Native injection: `a2` is the block hash, `a3` is its length.
pub fn get_header(buf: &mut [u8], block_hash: &[u8]) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        block_hash.as_ptr() as usize,
        block_hash.len() as u64,
        0,
        0,
        0,
        SYS_GET_HEADER,
    )
}

/// CKB RPC `get_header_by_number`.
///
/// Native injection: `a3` is the block number.
pub fn get_header_by_number(buf: &mut [u8], block_number: u64) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        0,
        block_number,
        0,
        0,
        0,
        SYS_GET_HEADER_BY_NUMBER,
    )
}

/// CKB RPC `get_block_hash`.
///
/// Native injection: `a3` is the block number. The hash is written into `buf`.
pub fn get_block_hash(buf: &mut [u8], block_number: u64) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        0,
        block_number,
        0,
        0,
        0,
        SYS_GET_BLOCK_HASH,
    )
}

/// Block hash from CKB RPC `get_transaction` (`tx_status.block_hash`).
///
/// Native injection: `a2` is the transaction hash, `a3` is its length.
pub fn get_transaction_block_hash(buf: &mut [u8], tx_hash: &[u8]) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        tx_hash.as_ptr() as usize,
        tx_hash.len() as u64,
        0,
        0,
        0,
        SYS_GET_TRANSACTION_BLOCK_HASH,
    )
}

/// ckb-indexer RPC `get_cells`.
///
/// Native injection registers:
/// `a2` is [`crate::indexer::SearchKey`] molecule bytes, `a3` their length,
/// `a4` [`Order`] (`0` asc, `1` desc), `a5` limit,
/// `a6` cursor whose first 4 bytes are the little-endian length of the `last_cursor` bytes that follow.
pub fn get_cells(
    buf: &mut [u8],
    search_key: &SearchKey,
    order: Order,
    limit: u64,
    last_cursor: &[u8],
) -> Result<usize, SysError> {
    let cursor_len = u32::try_from(last_cursor.len()).map_err(|_| SysError::Encoding)?;
    let mut cursor = Vec::with_capacity(4 + last_cursor.len());
    cursor.extend_from_slice(&cursor_len.to_le_bytes());
    cursor.extend_from_slice(last_cursor);
    let search_key = serde_molecule::to_vec(search_key, false).map_err(|_| SysError::Encoding)?;
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        search_key.as_ptr() as usize,
        search_key.len() as u64,
        order as u64,
        limit,
        cursor.as_ptr() as u64,
        SYS_GET_CELLS,
    )
}
