use crate::utils;
use alloc::vec;
use alloc::vec::Vec;
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
            let loaded_len = buf.len();
            data[..loaded_len].copy_from_slice(&buf);
            let len = syscall(&mut data[loaded_len..], loaded_len)?;
            debug_assert_eq!(len + loaded_len, actual_size);
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
    utils::syscall_branch!(find_out_point_by_type(&mut data, type_script.as_slice()))?;
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
        utils::syscall_branch!(find_cell_by_out_point(buf, out_point.as_slice()))
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
        utils::syscall_branch!(find_cell_data_by_out_point(buf, out_point.as_slice()))
    })
}

/// CKB net RPC `local_node_info`.
pub fn network() -> Result<Vec<u8>, SysError> {
    load_data(|buf, _offset| utils::syscall_branch!(network(buf)))
}

/// CKB RPC `get_live_cell`.
///
/// `with_data` asks the host to include the cell data in the returned bytes.
pub fn get_live_cell(out_point: OutPoint, with_data: bool) -> Result<Vec<u8>, SysError> {
    load_data(|buf, _offset| {
        utils::syscall_branch!(get_live_cell(buf, out_point.as_slice(), with_data))
    })
}

/// CKB RPC `get_header`.
pub fn get_header(block_hash: Byte32) -> Result<Header, SysError> {
    let data =
        load_data(|buf, _offset| utils::syscall_branch!(get_header(buf, block_hash.as_slice())))?;
    match HeaderReader::verify(&data, false) {
        Ok(()) => Ok(Header::new_unchecked(data.into())),
        Err(_err) => Err(SysError::Encoding),
    }
}

/// CKB RPC `get_header_by_number`.
pub fn get_header_by_number(block_number: u64) -> Result<Header, SysError> {
    let data =
        load_data(|buf, _offset| utils::syscall_branch!(get_header_by_number(buf, block_number)))?;
    match HeaderReader::verify(&data, false) {
        Ok(()) => Ok(Header::new_unchecked(data.into())),
        Err(_err) => Err(SysError::Encoding),
    }
}

/// CKB RPC `get_block_hash`.
pub fn get_block_hash(block_number: u64) -> Result<Byte32, SysError> {
    let mut data = [0u8; Byte32::TOTAL_SIZE];
    utils::syscall_branch!(get_block_hash(&mut data, block_number))?;
    match Byte32Reader::verify(&data, false) {
        Ok(()) => Ok(Byte32::new_unchecked(data.to_vec().into())),
        Err(_err) => Err(SysError::Encoding),
    }
}

/// Block hash of a committed transaction, from `get_transaction`'s `tx_status.block_hash`.
pub fn get_transaction_block_hash(tx_hash: Byte32) -> Result<Byte32, SysError> {
    let mut data = [0u8; Byte32::TOTAL_SIZE];
    utils::syscall_branch!(get_transaction_block_hash(&mut data, tx_hash.as_slice()))?;
    match Byte32Reader::verify(&data, false) {
        Ok(()) => Ok(Byte32::new_unchecked(data.to_vec().into())),
        Err(_err) => Err(SysError::Encoding),
    }
}

/// ckb-indexer RPC `get_cells`.
///
/// `order` is `0` for ascending and `1` for descending. `after` is the pagination cursor.
pub fn get_cells(
    search_key: &[u8],
    order: u64,
    limit: u64,
    after: &[u8],
) -> Result<Vec<u8>, SysError> {
    load_data(|buf, _offset| {
        utils::syscall_branch!(get_cells(buf, search_key, order, limit, after))
    })
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::utils;
    use ckb_std::ckb_types;
    use ckb_std::env;
    use ckb_types::packed::{Byte32, OutPoint, Script};
    use ckb_types::prelude::Entity;
    use std::sync::Mutex;
    use utils::{high_level, on_chain, syscalls, SysError};

    static ARGV_LOCK: Mutex<()> = Mutex::new(());

    fn with_argv<T>(argv: &'static [env::Arg], f: impl FnOnce() -> T) -> T {
        let _lock = ARGV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        unsafe { env::set_argv(argv) };
        struct Restore;
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe { env::set_argv(&[]) }
            }
        }
        let _restore = Restore;
        f()
    }

    fn assert_on_chain(result: Result<impl Sized, SysError>, number: u64) {
        assert_eq!(result.err(), Some(SysError::Unknown(number)));
    }

    #[test]
    fn find_out_point_by_type_branches() {
        let mut buf = [0u8; 64];
        let script = Script::default().as_slice().to_vec();
        assert_on_chain(
            on_chain::find_out_point_by_type(&mut buf, &script),
            syscalls::SYS_FIND_OUT_POINT_BY_TYPE,
        );

        let script = Script::default();
        assert_on_chain(
            with_argv(&[], || high_level::find_out_point_by_type(script)),
            syscalls::SYS_FIND_OUT_POINT_BY_TYPE,
        );
    }

    #[test]
    fn find_cell_by_out_point_branches() {
        let mut buf = [0u8; 64];
        let out_point = OutPoint::default().as_slice().to_vec();
        assert_on_chain(
            on_chain::find_cell_by_out_point(&mut buf, &out_point),
            syscalls::SYS_FIND_CELL_BY_OUT_POINT,
        );

        let out_point = OutPoint::default();
        assert_on_chain(
            with_argv(&[], || high_level::find_cell_by_out_point(out_point)),
            syscalls::SYS_FIND_CELL_BY_OUT_POINT,
        );
    }

    #[test]
    fn find_cell_data_by_out_point_branches() {
        let mut buf = [0u8; 64];
        let out_point = OutPoint::default().as_slice().to_vec();
        assert_on_chain(
            on_chain::find_cell_data_by_out_point(&mut buf, &out_point),
            syscalls::SYS_FIND_CELL_DATA_BY_OUT_POINT,
        );

        let out_point = OutPoint::default();
        assert_on_chain(
            with_argv(&[], || high_level::find_cell_data_by_out_point(out_point)),
            syscalls::SYS_FIND_CELL_DATA_BY_OUT_POINT,
        );
    }

    #[test]
    fn network_on_chain() {
        let mut buf = [0u8; 8];
        assert_on_chain(on_chain::network(&mut buf), syscalls::SYS_NETWORK);
        assert_on_chain(with_argv(&[], high_level::network), syscalls::SYS_NETWORK);
    }

    #[test]
    fn get_live_cell_on_chain() {
        let mut buf = [0u8; 8];
        assert_on_chain(
            on_chain::get_live_cell(&mut buf, &[], false),
            syscalls::SYS_GET_LIVE_CELL,
        );
        let out_point = OutPoint::default();
        assert_on_chain(
            with_argv(&[], || high_level::get_live_cell(out_point, false)),
            syscalls::SYS_GET_LIVE_CELL,
        );
    }

    #[test]
    fn get_header_on_chain() {
        let mut buf = [0u8; 8];
        assert_on_chain(
            on_chain::get_header(&mut buf, &[]),
            syscalls::SYS_GET_HEADER,
        );
        let block_hash = Byte32::default();
        assert_on_chain(
            with_argv(&[], || high_level::get_header(block_hash)),
            syscalls::SYS_GET_HEADER,
        );
    }

    #[test]
    fn get_header_by_number_on_chain() {
        let mut buf = [0u8; 8];
        assert_on_chain(
            on_chain::get_header_by_number(&mut buf, 0),
            syscalls::SYS_GET_HEADER_BY_NUMBER,
        );
        assert_on_chain(
            with_argv(&[], || high_level::get_header_by_number(0)),
            syscalls::SYS_GET_HEADER_BY_NUMBER,
        );
    }

    #[test]
    fn get_block_hash_on_chain() {
        let mut buf = [0u8; 8];
        assert_on_chain(
            on_chain::get_block_hash(&mut buf, 0),
            syscalls::SYS_GET_BLOCK_HASH,
        );
        assert_on_chain(
            with_argv(&[], || high_level::get_block_hash(0)),
            syscalls::SYS_GET_BLOCK_HASH,
        );
    }

    #[test]
    fn get_transaction_block_hash_on_chain() {
        let mut buf = [0u8; 8];
        assert_on_chain(
            on_chain::get_transaction_block_hash(&mut buf, &[]),
            syscalls::SYS_GET_TRANSACTION_BLOCK_HASH,
        );
        let tx_hash = Byte32::default();
        assert_on_chain(
            with_argv(&[], || high_level::get_transaction_block_hash(tx_hash)),
            syscalls::SYS_GET_TRANSACTION_BLOCK_HASH,
        );
    }

    #[test]
    fn get_cells_on_chain() {
        let mut buf = [0u8; 8];
        assert_on_chain(
            on_chain::get_cells(&mut buf, &[], 0, 1, &[]),
            syscalls::SYS_GET_CELLS,
        );
        assert_on_chain(
            with_argv(&[], || high_level::get_cells(&[], 0, 1, &[])),
            syscalls::SYS_GET_CELLS,
        );
    }
}
