//! Molecule layout of ckb-indexer's `get_cells` search key, and the molecule
//! values returned by `get_cells` and `get_live_cell`.
//!
//! Field order matches `IndexerSearchKey` / `IndexerSearchKeyFilter` in CKB
//! (`util/jsonrpc-types/src/indexer.rs`) and `ckb_sdk::rpc::ckb_indexer::SearchKey`.
//! [`Script`] and [`CellOutput`] are blockchain molecule tables. [`OutPoint`]
//! is a molecule struct. Their bytes equal `ckb_types::packed`.
//!
//! `serde_molecule` encodes each struct as a molecule table, `Option` as a
//! molecule option (absent is empty, present is the inner value), `Vec<u8>` as
//! `Bytes`, and a unit enum as the little-endian union item id of its variant.
//! `bool` is one byte, `0` or `1`. `[u64; 2]` is the indexer's half-open range
//! `[start, end)`. A field marked `struct_serde` is a molecule struct. `Vec` of
//! a table is a molecule vector (`dynvec_serde`).

use alloc::vec::Vec;
use ckb_std::ckb_types::{packed, prelude::*};
use serde::{Deserialize, Serialize};

/// ckb-indexer `SearchKey`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchKey {
    pub script: Script,
    pub script_type: ScriptType,
    /// Absent means the indexer default, prefix.
    pub script_search_mode: Option<SearchMode>,
    pub filter: Option<SearchKeyFilter>,
    /// Absent means the indexer default, include cell data.
    pub with_data: Option<bool>,
    /// Absent means the indexer default, do not group by transaction.
    pub group_by_transaction: Option<bool>,
}

/// ckb-indexer `SearchKeyFilter`. Every condition is optional.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchKeyFilter {
    /// The other script: type when [`ScriptType`] is lock, and lock when it is type.
    pub script: Option<Script>,
    pub script_len_range: Option<[u64; 2]>,
    pub output_data: Option<Vec<u8>>,
    /// Absent means the indexer default, prefix.
    pub output_data_filter_mode: Option<SearchMode>,
    pub output_data_len_range: Option<[u64; 2]>,
    pub output_capacity_range: Option<[u64; 2]>,
    pub block_range: Option<[u64; 2]>,
}

/// Blockchain molecule `Script`.
///
/// `hash_type` is CKB `ScriptHashType`: `0` data, `1` type, `2` data1, `4` data2.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Script {
    pub code_hash: [u8; 32],
    pub hash_type: u8,
    pub args: Vec<u8>,
}

impl From<packed::Script> for Script {
    fn from(script: packed::Script) -> Self {
        let mut code_hash = [0u8; 32];
        code_hash.copy_from_slice(script.code_hash().as_slice());
        let hash_type = script.hash_type().as_slice()[0];
        let args = script.args().raw_data().as_ref().to_vec();
        Self {
            code_hash,
            hash_type,
            args,
        }
    }
}

/// ckb-indexer `script_type`. Item ids: `Lock` = 0, `Type` = 1.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScriptType {
    #[default]
    Lock,
    Type,
}

/// ckb-indexer search mode. Item ids: `Prefix` = 0, `Exact` = 1, `Partial` = 2.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchMode {
    #[default]
    Prefix,
    Exact,
    Partial,
}

/// ckb-indexer `order`. Register values: `Asc` = 0, `Desc` = 1.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u64)]
pub enum Order {
    #[default]
    Asc = 0,
    Desc = 1,
}

/// Blockchain molecule `CellOutput`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CellOutput {
    pub capacity: u64,
    pub lock: Script,
    pub type_: Option<Script>,
}

impl From<packed::CellOutput> for CellOutput {
    fn from(output: packed::CellOutput) -> Self {
        Self {
            capacity: output.capacity().unpack(),
            lock: output.lock().into(),
            type_: output.type_().to_opt().map(Into::into),
        }
    }
}

/// Blockchain molecule `OutPoint`. It is a molecule struct.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OutPoint {
    pub tx_hash: [u8; 32],
    pub index: u32,
}

impl From<packed::OutPoint> for OutPoint {
    fn from(out_point: packed::OutPoint) -> Self {
        let mut tx_hash = [0u8; 32];
        tx_hash.copy_from_slice(out_point.tx_hash().as_slice());
        Self {
            tx_hash,
            index: out_point.index().unpack(),
        }
    }
}

/// `get_live_cell` result. An absent `output` is an unknown cell.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LiveCell {
    pub output: Option<CellOutput>,
    pub data: Option<Vec<u8>>,
    pub block_hash: Option<[u8; 32]>,
}

/// One cell in a `get_cells` page.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IndexerCell {
    pub output: CellOutput,
    pub output_data: Option<Vec<u8>>,
    #[serde(with = "serde_molecule::struct_serde")]
    pub out_point: OutPoint,
    pub block_number: u64,
    pub tx_index: u32,
}

/// One `get_cells` page. `last_cursor` is molecule `Bytes`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Pagination {
    #[serde(with = "serde_molecule::dynvec_serde")]
    pub objects: Vec<IndexerCell>,
    pub last_cursor: Vec<u8>,
}
