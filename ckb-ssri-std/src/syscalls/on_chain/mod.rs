use ckb_std::{
    ckb_constants::Source,
    ckb_types::{
        packed::{OutPoint, Script},
        prelude::*,
    },
    error::SysError,
    high_level::{self, QueryIter},
};

use crate::indexer::{Order, SearchKey};

mod context;
mod pagination;

use context::*;

/// On-chain adaptation of `find_out_point_by_type`.
///
/// The first code cell dep, then input, whose type script equals `type_script`.
pub fn find_out_point_by_type(buf: &mut [u8], type_script: &[u8]) -> Result<usize, SysError> {
    let type_script = Script::from_compatible_slice(type_script).map_err(|_| SysError::Encoding)?;
    let cell = load_celldep_and_input_cells()?
        .into_iter()
        .find(|cell| {
            cell.output
                .type_()
                .to_opt()
                .is_some_and(|script| script.as_slice() == type_script.as_slice())
        })
        .ok_or(SysError::ItemMissing)?;
    write_bytes(buf, cell.out_point.as_slice())
}

/// On-chain adaptation of `find_cell_by_out_point`.
///
/// The cell output for that out point in a code cell dep or an input.
pub fn find_cell_by_out_point(buf: &mut [u8], out_point: &[u8]) -> Result<usize, SysError> {
    let out_point = OutPoint::from_compatible_slice(out_point).map_err(|_| SysError::Encoding)?;
    let cell = load_celldep_and_input_cells()?
        .into_iter()
        .find(|cell| cell.out_point.as_slice() == out_point.as_slice())
        .ok_or(SysError::ItemMissing)?;
    write_bytes(buf, cell.output.as_slice())
}

/// On-chain adaptation of `find_cell_data_by_out_point`.
///
/// That cell's data in a code cell dep or an input.
pub fn find_cell_data_by_out_point(buf: &mut [u8], out_point: &[u8]) -> Result<usize, SysError> {
    let out_point = OutPoint::from_compatible_slice(out_point).map_err(|_| SysError::Encoding)?;
    let cells = load_celldep_and_input_cells()?;
    let cell = cells
        .iter()
        .find(|cell| cell.out_point.as_slice() == out_point.as_slice())
        .ok_or(SysError::ItemMissing)?;
    write_bytes(buf, &cell.data)
}

/// On-chain adaptation of `network`.
///
/// The first header dep whose raw hash is the mainnet or testnet genesis hash.
/// Any other chain writes `unknown`.
pub fn network(buf: &mut [u8]) -> Result<usize, SysError> {
    let tx = high_level::load_transaction()?.raw();
    let network = tx.header_deps().into_iter().find_map(|hash| {
        if hash.unpack() == MAINNET_GENESIS {
            Some(b"mainnet")
        } else if hash.unpack() == TESTNET_GENESIS {
            Some(b"testnet")
        } else {
            None
        }
    });
    write_bytes(buf, network.unwrap_or(b"unknown"))
}

/// On-chain adaptation of `get_live_cell`.
///
/// Molecule [`crate::indexer::LiveCell`] for that out point in a code cell dep or an input.
/// A missing cell has no output. `with_data` includes the cell data.
pub fn get_live_cell(buf: &mut [u8], out_point: &[u8], with_data: bool) -> Result<usize, SysError> {
    let out_point = OutPoint::from_compatible_slice(out_point).map_err(|_| SysError::Encoding)?;
    let cells = load_celldep_and_input_cells()?;
    let cell = cells
        .iter()
        .find(|cell| cell.out_point.as_slice() == out_point.as_slice());
    let encoded = serde_molecule::to_vec(&pagination::live_cell(cell, with_data), false)
        .map_err(|_| SysError::Encoding)?;
    write_bytes(buf, &encoded)
}

/// On-chain adaptation of `get_header`.
///
/// The header dep whose raw hash equals `block_hash`.
pub fn get_header(buf: &mut [u8], block_hash: &[u8]) -> Result<usize, SysError> {
    let tx = high_level::load_transaction()?.raw();
    let block_hash: [u8; 32] = block_hash.try_into().map_err(|_| SysError::Encoding)?;
    let index = tx
        .header_deps()
        .into_iter()
        .enumerate()
        .find_map(|(index, hash)| {
            if hash.unpack() == block_hash {
                Some(index)
            } else {
                None
            }
        })
        .ok_or(SysError::ItemMissing)?;
    let header = high_level::load_header(index, Source::HeaderDep)?;
    write_bytes(buf, header.as_slice())
}

/// On-chain adaptation of `get_header_by_number`.
///
/// The first header dep whose block number matches.
pub fn get_header_by_number(buf: &mut [u8], block_number: u64) -> Result<usize, SysError> {
    let header = QueryIter::new(high_level::load_header, Source::HeaderDep)
        .find(|header| header.raw().number().unpack() == block_number)
        .ok_or(SysError::ItemMissing)?;
    write_bytes(buf, header.as_slice())
}

/// On-chain adaptation of `get_block_hash`.
///
/// The raw header-dep hash of the first header with that block number.
pub fn get_block_hash(buf: &mut [u8], block_number: u64) -> Result<usize, SysError> {
    let header = QueryIter::new(high_level::load_header, Source::HeaderDep)
        .find(|header| header.raw().number().unpack() == block_number)
        .ok_or(SysError::ItemMissing)?;
    let hash = header.calc_header_hash().unpack();
    write_bytes(buf, &hash)
}

/// On-chain adaptation of `get_transaction_block_hash`.
///
/// The block hash of the first code cell dep or input whose out point uses `tx_hash`,
/// when that cell's header is in `header_deps`.
pub fn get_transaction_block_hash(buf: &mut [u8], tx_hash: &[u8]) -> Result<usize, SysError> {
    let tx_hash: [u8; 32] = tx_hash.try_into().map_err(|_| SysError::Encoding)?;
    let cells = load_celldep_and_input_cells()?;
    let cell = cells
        .iter()
        .find(|cell| cell.out_point.tx_hash().as_slice() == tx_hash.as_slice())
        .ok_or(SysError::ItemMissing)?;
    let hash = cell.block_hash.ok_or(SysError::ItemMissing)?;
    write_bytes(buf, &hash)
}

/// On-chain adaptation of `get_cells`.
///
/// `search_key` is [`SearchKey`] molecule bytes. The buffer receives the molecule
/// encoding of [`crate::indexer::Pagination`] for matching code cell deps, then inputs.
pub fn get_cells(
    buf: &mut [u8],
    search_key: &SearchKey,
    order: Order,
    limit: u64,
    last_cursor: &[u8],
) -> Result<usize, SysError> {
    let start = if last_cursor.is_empty() {
        0
    } else if let Ok(bytes) = <[u8; 8]>::try_from(last_cursor) {
        u64::from_le_bytes(bytes)
    } else {
        return Err(SysError::Encoding);
    };
    let cells = load_celldep_and_input_cells()?;
    let searched_cells = pagination::cells_page(&cells, search_key, order, limit, start)?;
    let encoded = serde_molecule::to_vec(&searched_cells, false).map_err(|_| SysError::Encoding)?;
    write_bytes(buf, &encoded)
}
