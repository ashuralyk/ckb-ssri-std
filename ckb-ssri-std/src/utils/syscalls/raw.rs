#[cfg(target_arch = "riscv64")]
use core::arch::asm;

use ckb_std::{ckb_constants::SYS_VM_VERSION, error::SysError};

#[cfg(target_arch = "riscv64")]
#[allow(clippy::too_many_arguments)]
/// Invoke a CKB VM syscall through `ecall`.
///
/// # Safety
///
/// Registers must match the syscall in `a7`. When that syscall writes through
/// `a0`, `a0` must be a valid pointer for the length in `a1`.
pub unsafe fn syscall(
    mut a0: u64,
    a1: u64,
    a2: u64,
    a3: u64,
    a4: u64,
    a5: u64,
    a6: u64,
    a7: u64,
) -> u64 {
    asm!(
        "ecall",
        inout("a0") a0,
        in("a1") a1,
        in("a2") a2,
        in("a3") a3,
        in("a4") a4,
        in("a5") a5,
        in("a6") a6,
        in("a7") a7
    );
    a0
}

#[cfg(not(target_arch = "riscv64"))]
#[allow(clippy::too_many_arguments)]
/// Host stub for [`syscall`]. It does not dereference the registers.
///
/// # Safety
///
/// Same contract as the RISC-V `ecall` syscall. This stub ignores the registers
/// and returns `u64::MAX`.
pub unsafe fn syscall(
    _a0: u64,
    _a1: u64,
    _a2: u64,
    _a3: u64,
    _a4: u64,
    _a5: u64,
    _a6: u64,
    _a7: u64,
) -> u64 {
    u64::MAX
}

pub fn vm_version() -> u64 {
    unsafe { syscall(0, 0, 0, 0, 0, 0, 0, SYS_VM_VERSION) }
}

/// Load data.
/// Return data length or syscall error.
#[allow(clippy::too_many_arguments)]
pub fn syscall_load(
    buf_ptr: *mut u8,
    len: usize,
    a2: usize,
    a3: u64,
    a4: u64,
    a5: u64,
    a6: u64,
    syscall_num: u64,
) -> Result<usize, SysError> {
    let mut actual_data_len = len;
    let len_ptr: *mut usize = &mut actual_data_len;
    let ret = unsafe {
        syscall(
            buf_ptr as u64,
            len_ptr as u64,
            a2 as u64,
            a3,
            a4,
            a5,
            a6,
            syscall_num,
        )
    };
    build_syscall_result(ret, len, actual_data_len)
}

fn build_syscall_result(
    errno: u64,
    load_len: usize,
    actual_data_len: usize,
) -> Result<usize, SysError> {
    use SysError::*;

    match errno {
        0 => {
            if actual_data_len > load_len {
                return Err(LengthNotEnough(actual_data_len));
            }
            Ok(actual_data_len)
        }
        1 => Err(IndexOutOfBound),
        2 => Err(ItemMissing),
        _ => Err(Unknown(errno)),
    }
}
