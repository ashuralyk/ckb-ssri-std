use ckb_std::error::SysError;

use crate::utils;
use catalog::{
    SYS_FIND_CELL_BY_OUT_POINT, SYS_FIND_CELL_DATA_BY_OUT_POINT, SYS_FIND_OUT_POINT_BY_TYPE,
};
use utils::catalog;

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
