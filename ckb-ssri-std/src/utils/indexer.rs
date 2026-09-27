//! Molecule layout of ckb-indexer's `get_cells` search key.
//!
//! Field order matches `IndexerSearchKey` / `IndexerSearchKeyFilter` in CKB
//! (`util/jsonrpc-types/src/indexer.rs`) and `ckb_sdk::rpc::ckb_indexer::SearchKey`.
//! [`Script`] is the blockchain molecule `Script` table: its bytes equal
//! `ckb_types::packed::Script`.
//!
//! `serde_molecule` encodes each struct as a molecule table, `Option` as a
//! molecule option (absent is empty, present is the inner value), `Vec<u8>` as
//! `Bytes`, and a unit enum as the little-endian union item id of its variant.
//! `bool` is one byte, `0` or `1`. `[u64; 2]` is the indexer's half-open range
//! `[start, end)`.

use alloc::vec::Vec;
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

impl From<ckb_std::ckb_types::packed::Script> for Script {
    fn from(script: ckb_std::ckb_types::packed::Script) -> Self {
        use ckb_std::ckb_types::prelude::Entity;

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

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::utils::{Script, ScriptType, SearchKey, SearchKeyFilter, SearchMode};
    use alloc::vec;
    use alloc::vec::Vec;
    use ckb_std::ckb_types::{packed, prelude::*};
    use serde_molecule::{from_slice, to_vec};

    fn table_fields(data: &[u8]) -> Vec<&[u8]> {
        let total = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
        assert_eq!(total, data.len());
        let first = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
        let count = first / 4 - 1;
        let mut offsets = Vec::with_capacity(count + 1);
        for index in 0..count {
            let start = 4 + index * 4;
            offsets.push(u32::from_le_bytes(data[start..start + 4].try_into().unwrap()) as usize);
        }
        offsets.push(data.len());
        offsets
            .windows(2)
            .map(|pair| &data[pair[0]..pair[1]])
            .collect()
    }

    fn sample_packed() -> packed::Script {
        packed::Script::new_builder()
            .code_hash([0x11u8; 32].pack())
            .hash_type(1u8)
            .args([0xabu8, 0xcd].as_slice().pack())
            .build()
    }

    #[test]
    fn script_matches_packed_molecule() {
        let packed = sample_packed();
        let encoded = to_vec(&Script::from(packed.clone()), false).unwrap();
        assert_eq!(encoded, packed.as_slice());
    }

    #[test]
    fn search_key_field_order_matches_indexer() {
        let packed = sample_packed();
        let filter_script = packed::Script::new_builder()
            .code_hash([0x22u8; 32].pack())
            .hash_type(0u8)
            .args([0x01u8].as_slice().pack())
            .build();
        let output_data = vec![0x7eu8, 0x7f];
        let key = SearchKey {
            script: packed.clone().into(),
            script_type: ScriptType::Type,
            script_search_mode: Some(SearchMode::Exact),
            filter: Some(SearchKeyFilter {
                script: Some(filter_script.clone().into()),
                script_len_range: Some([1, 2]),
                output_data: Some(output_data.clone()),
                output_data_filter_mode: Some(SearchMode::Partial),
                output_data_len_range: Some([3, 4]),
                output_capacity_range: Some([5, 6]),
                block_range: Some([7, 8]),
            }),
            with_data: Some(true),
            group_by_transaction: Some(false),
        };

        let encoded = to_vec(&key, false).unwrap();
        let fields = table_fields(&encoded);
        assert_eq!(fields.len(), 6);
        assert_eq!(fields[0], packed.as_slice());
        assert_eq!(fields[1], 1u32.to_le_bytes());
        assert_eq!(fields[2], 1u32.to_le_bytes());
        assert_eq!(fields[4], [1]);
        assert_eq!(fields[5], [0]);

        let filter = table_fields(fields[3]);
        assert_eq!(filter.len(), 7);
        assert_eq!(filter[0], filter_script.as_slice());
        assert_eq!(filter[1], range_bytes(1, 2));
        assert_eq!(filter[2], to_vec(&output_data, false).unwrap().as_slice());
        assert_eq!(filter[3], 2u32.to_le_bytes());
        assert_eq!(filter[4], range_bytes(3, 4));
        assert_eq!(filter[5], range_bytes(5, 6));
        assert_eq!(filter[6], range_bytes(7, 8));

        let decoded: SearchKey = from_slice(&encoded, false).unwrap();
        assert_eq!(decoded, key);
    }

    fn range_bytes(start: u64, end: u64) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&start.to_le_bytes());
        bytes[8..].copy_from_slice(&end.to_le_bytes());
        bytes
    }
}
