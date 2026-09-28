use alloc::{string::String, vec::Vec};
use ckb_std::{
    ckb_types::{packed::Script as PackedScript, prelude::*},
    error::SysError,
};
use serde::Serialize;

use crate::utils::{on_chain::LoadedCell, Script, ScriptType, SearchKey, SearchMode};

const HEX: &[u8; 16] = b"0123456789abcdef";

pub(crate) fn live_cell_json(
    cell: Option<&LoadedCell>,
    with_data: bool,
) -> Result<Vec<u8>, SysError> {
    let body = match cell {
        Some(cell) => JsonCellWithStatus {
            cell: Some(JsonCellInfo {
                data: cell_data_json(cell, with_data)?,
                output: json_output(&cell.output)?,
            }),
            status: "live",
            block_hash: cell.block_hash.map(|hash| hex_bytes(&hash)),
        },
        None => JsonCellWithStatus {
            cell: None,
            status: "unknown",
            block_hash: None,
        },
    };
    serde_json::to_vec(&body).map_err(|_| SysError::Encoding)
}

pub(crate) fn cells_page(
    cells: &[LoadedCell],
    key: &SearchKey,
    order: u64,
    limit: u64,
    start: u64,
) -> Result<Vec<u8>, SysError> {
    let descending = match order {
        0 => false,
        1 => true,
        _ => return Err(SysError::IndexOutOfBound),
    };
    let mut matched: Vec<&LoadedCell> = cells
        .iter()
        .filter(|cell| cell_matches(key, cell))
        .collect();
    if descending {
        matched.reverse();
    }
    let total = matched.len() as u64;
    let (page, next) = page_window(&matched, start, limit, total);
    let with_data = key.with_data.unwrap_or(true);
    let mut objects = Vec::with_capacity(page.len());
    for cell in page {
        objects.push(json_indexer_cell(cell, with_data)?);
    }
    let body = JsonPagination {
        objects,
        last_cursor: if next >= total {
            String::from("0x")
        } else {
            hex_le_u64(next)
        },
    };
    serde_json::to_vec(&body).map_err(|_| SysError::Encoding)
}

fn page_window<'a>(
    matched: &[&'a LoadedCell],
    start: u64,
    limit: u64,
    total: u64,
) -> (Vec<&'a LoadedCell>, u64) {
    if start >= total {
        return (Vec::new(), total);
    }
    let start_index = start as usize;
    let end = if limit == 0 {
        start_index
    } else {
        start_index
            .saturating_add(limit as usize)
            .min(matched.len())
    };
    (matched[start_index..end].to_vec(), end as u64)
}

fn cell_matches(key: &SearchKey, cell: &LoadedCell) -> bool {
    let mode = key.script_search_mode.unwrap_or(SearchMode::Prefix);
    if !primary_matches(key.script_type, mode, &key.script, cell) {
        return false;
    }
    let Some(filter) = &key.filter else {
        return true;
    };
    if let Some(script) = &filter.script {
        if !secondary_matches(key.script_type, script, cell) {
            return false;
        }
    }
    if let Some([start, end]) = filter.output_capacity_range {
        let capacity: u64 = cell.output.capacity().unpack();
        if capacity < start || capacity >= end {
            return false;
        }
    }
    if let Some([start, end]) = filter.output_data_len_range {
        let len = cell.data.len() as u64;
        if len < start || len >= end {
            return false;
        }
    }
    if let Some(output_data) = &filter.output_data {
        let mode = filter.output_data_filter_mode.unwrap_or(SearchMode::Prefix);
        if !bytes_match(mode, output_data, &cell.data) {
            return false;
        }
    }
    true
}

fn primary_matches(
    script_type: ScriptType,
    mode: SearchMode,
    query: &Script,
    cell: &LoadedCell,
) -> bool {
    match script_type {
        ScriptType::Lock => script_matches(mode, query, &cell.output.lock()),
        ScriptType::Type => cell
            .output
            .type_()
            .to_opt()
            .is_some_and(|script| script_matches(mode, query, &script)),
    }
}

fn secondary_matches(script_type: ScriptType, query: &Script, cell: &LoadedCell) -> bool {
    match script_type {
        ScriptType::Lock => cell
            .output
            .type_()
            .to_opt()
            .is_some_and(|script| script_matches(SearchMode::Prefix, query, &script)),
        ScriptType::Type => script_matches(SearchMode::Prefix, query, &cell.output.lock()),
    }
}

fn script_matches(mode: SearchMode, query: &Script, cell: &PackedScript) -> bool {
    if cell.code_hash().as_slice() != query.code_hash {
        return false;
    }
    if cell.hash_type().as_slice()[0] != query.hash_type {
        return false;
    }
    bytes_match(mode, &query.args, cell.args().raw_data().as_ref())
}

fn bytes_match(mode: SearchMode, query: &[u8], cell: &[u8]) -> bool {
    match mode {
        SearchMode::Exact => cell == query,
        SearchMode::Prefix => cell.starts_with(query),
        SearchMode::Partial => {
            if query.is_empty() {
                true
            } else {
                cell.windows(query.len()).any(|window| window == query)
            }
        }
    }
}

fn json_indexer_cell(cell: &LoadedCell, with_data: bool) -> Result<JsonIndexerCell, SysError> {
    Ok(JsonIndexerCell {
        output: json_output(&cell.output)?,
        output_data: if with_data {
            Some(hex_bytes(&cell.data))
        } else {
            None
        },
        out_point: json_out_point(cell)?,
        block_number: hex_u64(cell.block_number.unwrap_or(0)),
        tx_index: hex_u64(0),
    })
}

fn cell_data_json(cell: &LoadedCell, with_data: bool) -> Result<Option<JsonCellData>, SysError> {
    if !with_data {
        return Ok(None);
    }
    Ok(Some(JsonCellData {
        content: hex_bytes(&cell.data),
        hash: hex_bytes(&ckb_hash::blake2b_256(&cell.data)),
    }))
}

fn json_output(
    output: &ckb_std::ckb_types::packed::CellOutput,
) -> Result<JsonCellOutput, SysError> {
    Ok(JsonCellOutput {
        capacity: hex_u64(output.capacity().unpack()),
        lock: json_script(&output.lock())?,
        type_script: match output.type_().to_opt() {
            Some(script) => Some(json_script(&script)?),
            None => None,
        },
    })
}

fn json_script(script: &PackedScript) -> Result<JsonScript, SysError> {
    Ok(JsonScript {
        code_hash: hex_bytes(script.code_hash().as_slice()),
        hash_type: hash_type_name(script.hash_type().as_slice()[0])?,
        args: hex_bytes(script.args().raw_data().as_ref()),
    })
}

fn json_out_point(cell: &LoadedCell) -> Result<JsonOutPoint, SysError> {
    let index: u32 = cell.out_point.index().unpack();
    Ok(JsonOutPoint {
        tx_hash: hex_bytes(cell.out_point.tx_hash().as_slice()),
        index: hex_u64(u64::from(index)),
    })
}

fn hash_type_name(hash_type: u8) -> Result<String, SysError> {
    match hash_type {
        0 => Ok(String::from("data")),
        1 => Ok(String::from("type")),
        value if value % 2 == 0 => Ok(alloc::format!("data{}", value >> 1)),
        _ => Err(SysError::Encoding),
    }
}

fn hex_u64(value: u64) -> String {
    alloc::format!("0x{value:x}")
}

fn hex_le_u64(value: u64) -> String {
    hex_bytes(&value.to_le_bytes())
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(2 + bytes.len() * 2);
    out.push_str("0x");
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[derive(Serialize)]
struct JsonCellWithStatus {
    cell: Option<JsonCellInfo>,
    status: &'static str,
    block_hash: Option<String>,
}

#[derive(Serialize)]
struct JsonCellInfo {
    data: Option<JsonCellData>,
    output: JsonCellOutput,
}

#[derive(Serialize)]
struct JsonCellData {
    content: String,
    hash: String,
}

#[derive(Serialize)]
struct JsonCellOutput {
    capacity: String,
    lock: JsonScript,
    #[serde(rename = "type")]
    type_script: Option<JsonScript>,
}

#[derive(Serialize)]
struct JsonScript {
    code_hash: String,
    hash_type: String,
    args: String,
}

#[derive(Serialize)]
struct JsonOutPoint {
    tx_hash: String,
    index: String,
}

#[derive(Serialize)]
struct JsonIndexerCell {
    output: JsonCellOutput,
    output_data: Option<String>,
    out_point: JsonOutPoint,
    block_number: String,
    tx_index: String,
}

#[derive(Serialize)]
struct JsonPagination {
    objects: Vec<JsonIndexerCell>,
    last_cursor: String,
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::{cells_page, hex_bytes, live_cell_json};
    use crate::utils::on_chain::LoadedCell;
    use crate::utils::SysError;
    use crate::utils::{Script, ScriptType, SearchKey, SearchKeyFilter, SearchMode};
    use alloc::vec;
    use alloc::vec::Vec;
    use ckb_std::ckb_types::{packed, prelude::*};
    use serde_json::Value;

    fn script(code_hash: u8, hash_type: u8, args: &[u8]) -> packed::Script {
        packed::Script::new_builder()
            .code_hash([code_hash; 32].pack())
            .hash_type(hash_type)
            .args(args.pack())
            .build()
    }

    fn indexer_script(packed_script: &packed::Script) -> Script {
        packed_script.clone().into()
    }

    fn cell(
        tx: u8,
        index: u32,
        lock: packed::Script,
        type_script: Option<packed::Script>,
        capacity: u64,
        data: &[u8],
        block_number: Option<u64>,
    ) -> LoadedCell {
        let mut builder = packed::CellOutput::new_builder()
            .capacity(capacity)
            .lock(lock);
        if let Some(type_script) = type_script {
            builder = builder.type_(Some(type_script).pack());
        }
        LoadedCell {
            out_point: packed::OutPoint::new_builder()
                .tx_hash([tx; 32].pack())
                .index(index)
                .build(),
            output: builder.build(),
            data: data.to_vec(),
            block_hash: block_number.map(|number| [number as u8; 32]),
            block_number,
        }
    }

    fn sample_cells() -> Vec<LoadedCell> {
        let lock_a = script(0x11, 0, &[0xaa, 0x01]);
        let lock_b = script(0x11, 0, &[0xaa, 0x02, 0x03]);
        let type_a = script(0x22, 1, &[0x10]);
        vec![
            cell(1, 0, lock_a, Some(type_a), 100, &[0x7e, 0x7f], Some(8)),
            cell(2, 1, lock_b, None, 250, &[0x01], None),
        ]
    }

    fn parse(bytes: &[u8]) -> Value {
        serde_json::from_slice(bytes).unwrap()
    }

    #[test]
    fn live_cell_json_shapes() {
        let cells = sample_cells();
        let live = parse(&live_cell_json(Some(&cells[0]), true).unwrap());
        assert_eq!(live["status"], "live");
        assert_eq!(
            live["block_hash"],
            "0x0808080808080808080808080808080808080808080808080808080808080808"
        );
        assert_eq!(live["cell"]["data"]["content"], "0x7e7f");
        assert_eq!(
            live["cell"]["data"]["hash"],
            hex_bytes(&ckb_hash::blake2b_256([0x7eu8, 0x7f]))
        );
        assert_eq!(live["cell"]["output"]["capacity"], "0x64");
        assert_eq!(live["cell"]["output"]["lock"]["hash_type"], "data");
        assert_eq!(live["cell"]["output"]["type"]["hash_type"], "type");

        let without_data = parse(&live_cell_json(Some(&cells[0]), false).unwrap());
        assert!(without_data["cell"]["data"].is_null());

        let unknown = parse(&live_cell_json(None, true).unwrap());
        assert_eq!(unknown["status"], "unknown");
        assert!(unknown["cell"].is_null());
        assert!(unknown["block_hash"].is_null());
    }

    #[test]
    fn get_cells_filters_order_and_cursor() {
        let cells = sample_cells();
        let lock = indexer_script(&script(0x11, 0, &[0xaa]));
        let key = SearchKey {
            script: lock,
            script_type: ScriptType::Lock,
            script_search_mode: Some(SearchMode::Prefix),
            filter: Some(SearchKeyFilter {
                script_len_range: Some([0, 1]),
                block_range: Some([100, 101]),
                ..SearchKeyFilter::default()
            }),
            with_data: Some(false),
            group_by_transaction: Some(true),
        };

        let page = parse(&cells_page(&cells, &key, 0, 1, 0).unwrap());
        assert_eq!(page["objects"].as_array().unwrap().len(), 1);
        assert!(page["objects"][0]["output_data"].is_null());
        assert_eq!(page["objects"][0]["block_number"], "0x8");
        assert_eq!(page["objects"][0]["tx_index"], "0x0");
        assert_eq!(page["objects"][0]["out_point"]["index"], "0x0");
        assert_eq!(page["last_cursor"], "0x0100000000000000");

        let next = parse(&cells_page(&cells, &key, 0, 1, 1).unwrap());
        assert_eq!(next["objects"][0]["out_point"]["index"], "0x1");
        assert_eq!(next["objects"][0]["block_number"], "0x0");
        assert_eq!(next["last_cursor"], "0x");

        let desc = parse(&cells_page(&cells, &key, 1, 1, 0).unwrap());
        assert_eq!(desc["objects"][0]["out_point"]["index"], "0x1");

        let empty = parse(&cells_page(&cells, &key, 0, 0, 0).unwrap());
        assert!(empty["objects"].as_array().unwrap().is_empty());
        assert_eq!(empty["last_cursor"], "0x0000000000000000");
    }

    #[test]
    fn search_modes_and_ranges() {
        let cells = sample_cells();
        let exact = SearchKey {
            script: indexer_script(&script(0x11, 0, &[0xaa, 0x01])),
            script_type: ScriptType::Lock,
            script_search_mode: Some(SearchMode::Exact),
            ..SearchKey::default()
        };
        let exact_page = parse(&cells_page(&cells, &exact, 0, 10, 0).unwrap());
        assert_eq!(exact_page["objects"].as_array().unwrap().len(), 1);
        assert_eq!(exact_page["objects"][0]["output_data"], "0x7e7f");

        let partial = SearchKey {
            script: indexer_script(&script(0x11, 0, &[0x02, 0x03])),
            script_type: ScriptType::Lock,
            script_search_mode: Some(SearchMode::Partial),
            filter: Some(SearchKeyFilter {
                output_capacity_range: Some([200, 300]),
                output_data_len_range: Some([1, 2]),
                output_data: Some(vec![0x01]),
                output_data_filter_mode: Some(SearchMode::Exact),
                ..SearchKeyFilter::default()
            }),
            ..SearchKey::default()
        };
        let partial_page = parse(&cells_page(&cells, &partial, 0, 10, 0).unwrap());
        assert_eq!(partial_page["objects"].as_array().unwrap().len(), 1);
        assert_eq!(partial_page["objects"][0]["out_point"]["index"], "0x1");

        let type_key = SearchKey {
            script: indexer_script(&script(0x22, 1, &[0x10])),
            script_type: ScriptType::Type,
            script_search_mode: Some(SearchMode::Exact),
            filter: Some(SearchKeyFilter {
                script: Some(indexer_script(&script(0x11, 0, &[0xaa]))),
                ..SearchKeyFilter::default()
            }),
            ..SearchKey::default()
        };
        let type_page = parse(&cells_page(&cells, &type_key, 0, 10, 0).unwrap());
        assert_eq!(type_page["objects"].as_array().unwrap().len(), 1);

        assert_eq!(
            cells_page(&cells, &exact, 2, 1, 0).unwrap_err(),
            SysError::IndexOutOfBound
        );
    }
}
