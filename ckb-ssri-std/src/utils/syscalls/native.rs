use ckb_std::error::SysError;

use crate::utils;
use catalog::{
    SYS_FIND_CELL_BY_OUT_POINT, SYS_FIND_CELL_DATA_BY_OUT_POINT, SYS_FIND_OUT_POINT_BY_TYPE,
};
use raw::syscall_load;
use utils::{catalog, raw};

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
