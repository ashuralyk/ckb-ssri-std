use crate::utils::{syscall_branch, LiveCell, Order, Pagination, SearchKey};
use alloc::{vec, vec::Vec};
use ckb_std::{
    ckb_types::{
        packed::{
            Byte32, Byte32Reader, CellOutput, CellOutputReader, Header, HeaderReader, OutPoint,
            OutPointReader, Script,
        },
        prelude::*,
    },
    error::SysError,
    high_level::BUF_SIZE,
};

/// Common method to fully load data from syscall
fn load_data<F: Fn(&mut [u8], usize) -> Result<usize, SysError>>(
    syscall: F,
) -> Result<Vec<u8>, SysError> {
    let mut buf = [0u8; BUF_SIZE];
    match syscall(&mut buf, 0) {
        Ok(len) => Ok(buf[..len].to_vec()),
        Err(SysError::LengthNotEnough(actual_size)) => {
            let mut data = vec![0; actual_size];
            let len = syscall(&mut data, 0)?;
            if len != actual_size {
                return Err(SysError::Encoding);
            }
            Ok(data)
        }
        Err(err) => Err(err),
    }
}

/// Find an OutPoint by searching for a cell with a specific type script
///
/// Searches the transaction for a cell that matches the given type script
/// and returns its OutPoint if found.
///
/// # Arguments
///
/// * `type_script` - The Script to search for as a cell's type script
///
/// # Returns
///
/// * `Ok(OutPoint)` - The OutPoint of the first matching cell
/// * `Err(SysError)` - A system error if the operation fails
///
/// # Example
///
/// ```ignore
/// let out_point = find_out_point_by_type(type_script).unwrap();
/// ```
///
/// # Errors
///
/// * Returns `SysError::ItemMissing` if no cell with matching type script is found
/// * Returns `SysError::Encoding` if the OutPoint data is malformed
///
/// # Panics
///
/// This function can panic if the underlying data is too large,
/// potentially causing an out-of-memory error.
pub fn find_out_point_by_type(type_script: Script) -> Result<OutPoint, SysError> {
    let mut data = [0u8; OutPoint::TOTAL_SIZE];
    syscall_branch!(find_out_point_by_type(&mut data, type_script.as_slice()))?;
    match OutPointReader::verify(&data, false) {
        Ok(()) => Ok(OutPoint::new_unchecked(data.to_vec().into())),
        Err(_err) => Err(SysError::Encoding),
    }
}

/// Find a cell by its OutPoint
///
/// Retrieves the CellOutput of a cell identified by the given OutPoint.
///
/// # Arguments
///
/// * `out_point` - The OutPoint identifying the cell to find
///
/// # Returns
///
/// * `Ok(CellOutput)` - The cell's output data if found
/// * `Err(SysError)` - A system error if the operation fails
///
/// # Example
///
/// ```ignore
/// let out_point = OutPoint::new(...);
/// let cell_output = find_cell_by_out_point(out_point).unwrap();
/// ```
///
/// # Errors
///
/// * Returns `SysError::ItemMissing` if the cell cannot be found
/// * Returns `SysError::Encoding` if the CellOutput data is malformed
///
/// # Panics
///
/// This function can panic if the underlying data is too large,
/// potentially causing an out-of-memory error.
pub fn find_cell_by_out_point(out_point: OutPoint) -> Result<CellOutput, SysError> {
    let data = load_data(|buf, _offset| {
        syscall_branch!(find_cell_by_out_point(buf, out_point.as_slice()))
    })?;

    match CellOutputReader::verify(&data, false) {
        Ok(()) => Ok(CellOutput::new_unchecked(data.into())),
        Err(_err) => Err(SysError::Encoding),
    }
}

/// Find cell data by OutPoint
///
/// Retrieves the data contained in a cell identified by the given OutPoint.
///
/// # Arguments
///
/// * `out_point` - The OutPoint identifying the cell whose data to retrieve
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - The cell's data as a byte vector if found
/// * `Err(SysError)` - A system error if the operation fails
///
/// # Example
///
/// ```ignore
/// let out_point = OutPoint::new(...);
/// let data = find_cell_data_by_out_point(out_point).unwrap();
/// ```
///
/// # Errors
///
/// * Returns `SysError::ItemMissing` if the cell cannot be found
/// * Returns `SysError::LengthNotEnough` if the data buffer is too small
///
/// # Panics
///
/// This function can panic if the underlying data is too large,
/// potentially causing an out-of-memory error.
pub fn find_cell_data_by_out_point(out_point: OutPoint) -> Result<Vec<u8>, SysError> {
    load_data(|buf, _offset| {
        syscall_branch!(find_cell_data_by_out_point(buf, out_point.as_slice()))
    })
}

/// CKB net RPC `local_node_info`.
pub fn network() -> Result<Vec<u8>, SysError> {
    load_data(|buf, _offset| syscall_branch!(network(buf)))
}

/// CKB RPC `get_live_cell`.
///
/// Returns a molecule [`LiveCell`]. `with_data` includes the cell data.
pub fn get_live_cell(out_point: OutPoint, with_data: bool) -> Result<LiveCell, SysError> {
    let data = load_data(|buf, _offset| {
        syscall_branch!(get_live_cell(buf, out_point.as_slice(), with_data))
    })?;
    serde_molecule::from_slice(&data, false).map_err(|_| SysError::Encoding)
}

/// CKB RPC `get_header`.
pub fn get_header(block_hash: Byte32) -> Result<Header, SysError> {
    let data = load_data(|buf, _offset| syscall_branch!(get_header(buf, block_hash.as_slice())))?;
    match HeaderReader::verify(&data, false) {
        Ok(()) => Ok(Header::new_unchecked(data.into())),
        Err(_err) => Err(SysError::Encoding),
    }
}

/// CKB RPC `get_header_by_number`.
pub fn get_header_by_number(block_number: u64) -> Result<Header, SysError> {
    let data = load_data(|buf, _offset| syscall_branch!(get_header_by_number(buf, block_number)))?;
    match HeaderReader::verify(&data, false) {
        Ok(()) => Ok(Header::new_unchecked(data.into())),
        Err(_err) => Err(SysError::Encoding),
    }
}

/// CKB RPC `get_block_hash`.
pub fn get_block_hash(block_number: u64) -> Result<Byte32, SysError> {
    let mut data = [0u8; Byte32::TOTAL_SIZE];
    syscall_branch!(get_block_hash(&mut data, block_number))?;
    match Byte32Reader::verify(&data, false) {
        Ok(()) => Ok(Byte32::new_unchecked(data.to_vec().into())),
        Err(_err) => Err(SysError::Encoding),
    }
}

/// Block hash of a committed transaction, from `get_transaction`'s `tx_status.block_hash`.
pub fn get_transaction_block_hash(tx_hash: Byte32) -> Result<Byte32, SysError> {
    let mut data = [0u8; Byte32::TOTAL_SIZE];
    syscall_branch!(get_transaction_block_hash(&mut data, tx_hash.as_slice()))?;
    match Byte32Reader::verify(&data, false) {
        Ok(()) => Ok(Byte32::new_unchecked(data.to_vec().into())),
        Err(_err) => Err(SysError::Encoding),
    }
}

/// ckb-indexer RPC `get_cells`.
///
/// Returns a [`Pagination`] of molecule cells. `search_key` is the molecule table
/// in [`SearchKey`]. `order` is [`Order::Asc`] or [`Order::Desc`].
/// `after` is the pagination cursor.
pub fn get_cells(
    search_key: &SearchKey,
    order: Order,
    limit: u64,
    after: u64,
) -> Result<Pagination, SysError> {
    let data =
        load_data(|buf, _offset| syscall_branch!(get_cells(buf, search_key, order, limit, after)))?;
    serde_molecule::from_slice(&data, false).map_err(|_| SysError::Encoding)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::load_data;
    use crate::utils::SysError;
    use alloc::vec;
    use core::cell::Cell;

    #[test]
    fn load_data_retries_into_a_full_buffer() {
        let payload = vec![7u8; 300];
        let calls = Cell::new(0usize);
        let data = load_data(|buf, offset| {
            let call = calls.get() + 1;
            calls.set(call);
            assert_eq!(offset, 0);
            if call == 1 {
                assert!(buf.len() < payload.len());
                buf.copy_from_slice(&payload[..buf.len()]);
                Err(SysError::LengthNotEnough(payload.len()))
            } else {
                assert_eq!(buf.len(), payload.len());
                buf.copy_from_slice(&payload);
                Ok(payload.len())
            }
        })
        .unwrap();
        assert_eq!(calls.get(), 2);
        assert_eq!(data, payload);
    }
}
