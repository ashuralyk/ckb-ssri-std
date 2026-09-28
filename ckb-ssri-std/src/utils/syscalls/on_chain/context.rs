use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{
        packed::{CellOutput, OutPoint},
        prelude::*,
    },
    error::SysError,
    high_level,
};

/// `cell_deps` byte for a dep group. Members are not searched, and the entry does not take an index.
pub const DEP_GROUP: u8 = 1;

pub const MAINNET_GENESIS: [u8; 32] = [
    0x92, 0xb1, 0x97, 0xaa, 0x1f, 0xba, 0x0f, 0x63, 0x63, 0x39, 0x22, 0xc6, 0x1c, 0x92, 0x37, 0x5c,
    0x9c, 0x07, 0x4a, 0x93, 0xe8, 0x59, 0x63, 0x55, 0x4f, 0x54, 0x99, 0xfe, 0x14, 0x50, 0xd0, 0xe5,
];

pub const TESTNET_GENESIS: [u8; 32] = [
    0x10, 0x63, 0x9e, 0x08, 0x95, 0x50, 0x2b, 0x56, 0x88, 0xa6, 0xbe, 0x8c, 0xf6, 0x94, 0x60, 0xd7,
    0x65, 0x41, 0xbf, 0xa4, 0x82, 0x16, 0x29, 0xd8, 0x6d, 0x62, 0xba, 0x0a, 0xae, 0x3f, 0x96, 0x06,
];

/// A cell from a code cell dep or an input, in that search order.
pub struct LoadedCell {
    pub out_point: OutPoint,
    pub output: CellOutput,
    pub data: Vec<u8>,
    pub block_hash: Option<[u8; 32]>,
    pub block_number: Option<u64>,
}

impl LoadedCell {
    pub fn new(out_point: OutPoint, index: usize, source: Source) -> Result<Self, SysError> {
        let header = match high_level::load_header(index, source) {
            Ok(header) => Some(header),
            Err(SysError::ItemMissing) => None,
            Err(err) => return Err(err),
        };
        Ok(LoadedCell {
            out_point,
            output: high_level::load_cell(index, source)?,
            data: high_level::load_cell_data(index, source)?,
            block_hash: header
                .as_ref()
                .map(|header| header.calc_header_hash().unpack()),
            block_number: header.as_ref().map(|header| header.raw().number().unpack()),
        })
    }
}

/// Copy `data` into `buf`. A short buffer is filled with the prefix and reports the full length.
pub fn write_bytes(buf: &mut [u8], data: &[u8]) -> Result<usize, SysError> {
    if data.len() > buf.len() {
        buf.copy_from_slice(&data[..buf.len()]);
        Err(SysError::LengthNotEnough(data.len()))
    } else {
        buf[..data.len()].copy_from_slice(data);
        Ok(data.len())
    }
}

pub fn load_celldep_and_input_cells() -> Result<Vec<LoadedCell>, SysError> {
    let tx = high_level::load_transaction()?.raw();
    let mut cells = Vec::new();
    let celldeps = tx.cell_deps().into_iter().enumerate();
    let inputs = tx.inputs().into_iter().enumerate();
    for (index, dep) in celldeps {
        if dep.dep_type().as_slice()[0] == DEP_GROUP {
            break;
        }
        cells.push(LoadedCell::new(dep.out_point(), index, Source::CellDep)?);
    }
    for (index, input) in inputs {
        cells.push(LoadedCell::new(
            input.previous_output(),
            index,
            Source::Input,
        )?);
    }
    Ok(cells)
}
