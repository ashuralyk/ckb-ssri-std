use crate::utils;
use alloc::vec;
use alloc::vec::Vec;
use ckb_std::{
    ckb_types::{
        packed::{CellOutput, CellOutputReader, OutPoint, OutPointReader, Script},
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

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::utils;
    use ckb_std::ckb_types;
    use ckb_std::env;
    use ckb_types::packed::{OutPoint, Script};
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
}
