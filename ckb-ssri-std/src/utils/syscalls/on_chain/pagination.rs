use alloc::vec::Vec;
use ckb_std::{
    ckb_types::{packed::Script as PackedScript, prelude::*},
    error::SysError,
};

use crate::utils::{
    indexer::{
        IndexerCell, LiveCell, Order, Pagination, Script, ScriptType, SearchKey, SearchMode,
    },
    syscalls::on_chain::context::LoadedCell,
};

impl TryFrom<(&LoadedCell, bool)> for IndexerCell {
    type Error = SysError;

    fn try_from((cell, with_data): (&LoadedCell, bool)) -> Result<Self, SysError> {
        Ok(Self {
            output: cell.output.clone().into(),
            output_data: with_data.then(|| cell.data.clone()),
            out_point: cell.out_point.clone().into(),
            block_number: cell.block_number.unwrap_or(0),
            tx_index: 0,
        })
    }
}

pub fn live_cell(cell: Option<&LoadedCell>, with_data: bool) -> LiveCell {
    match cell {
        Some(cell) => LiveCell {
            output: Some(cell.output.clone().into()),
            data: with_data.then(|| cell.data.clone()),
            block_hash: cell.block_hash,
        },
        None => LiveCell {
            output: None,
            data: None,
            block_hash: None,
        },
    }
}

pub fn cells_page(
    cells: &[LoadedCell],
    key: &SearchKey,
    order: Order,
    limit: u64,
    start: u64,
) -> Result<Pagination, SysError> {
    let mut matched: Vec<&LoadedCell> = cells
        .iter()
        .filter(|cell| cell_matches(key, cell))
        .collect();
    if matches!(order, Order::Desc) {
        matched.reverse();
    }
    let total = matched.len() as u64;
    let (page, next) = page_window(&matched, start, limit, total);
    let with_data = key.with_data.unwrap_or(true);
    let mut objects = Vec::with_capacity(page.len());
    for cell in page {
        objects.push(IndexerCell::try_from((cell, with_data))?);
    }
    Ok(Pagination {
        objects,
        last_cursor: if next >= total {
            Vec::new()
        } else {
            next.to_le_bytes().to_vec()
        },
    })
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
    let primary = match key.script_type {
        ScriptType::Lock => script_matches(mode, &key.script, &cell.output.lock()),
        ScriptType::Type => cell
            .output
            .type_()
            .to_opt()
            .is_some_and(|script| script_matches(mode, &key.script, &script)),
    };
    if !primary {
        return false;
    }
    let Some(filter) = &key.filter else {
        return true;
    };
    if let Some(script) = &filter.script {
        let secondary = match key.script_type {
            ScriptType::Lock => cell.output.type_().to_opt().is_some_and(|type_script| {
                script_matches(SearchMode::Prefix, script, &type_script)
            }),
            ScriptType::Type => script_matches(SearchMode::Prefix, script, &cell.output.lock()),
        };
        if !secondary {
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
        let matched = match mode {
            SearchMode::Exact => cell.data == *output_data,
            SearchMode::Prefix => cell.data.starts_with(output_data),
            SearchMode::Partial => {
                output_data.is_empty()
                    || cell
                        .data
                        .windows(output_data.len())
                        .any(|window| window == output_data)
            }
        };
        if !matched {
            return false;
        }
    }
    true
}

fn script_matches(mode: SearchMode, query: &Script, cell: &PackedScript) -> bool {
    if cell.code_hash().as_slice() != query.code_hash {
        return false;
    }
    if cell.hash_type().as_slice()[0] != query.hash_type {
        return false;
    }
    let args = cell.args().raw_data();
    let cell_args = args.as_ref();
    match mode {
        SearchMode::Exact => cell_args == query.args,
        SearchMode::Prefix => cell_args.starts_with(&query.args),
        SearchMode::Partial => {
            query.args.is_empty()
                || cell_args
                    .windows(query.args.len())
                    .any(|window| window == query.args)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::{vec, vec::Vec};
    use ckb_std::ckb_types::{packed, prelude::*};

    use super::{cells_page, live_cell};

    use crate::utils::{
        indexer::{
            CellOutput, LiveCell, Order, OutPoint, Pagination, Script, ScriptType, SearchKey,
            SearchKeyFilter, SearchMode,
        },
        syscalls::on_chain::context::LoadedCell,
    };

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

    #[test]
    fn live_cell_shapes() {
        let cells = sample_cells();
        let live = live_cell(Some(&cells[0]), true);
        assert_eq!(
            live.output.as_ref().unwrap(),
            &cells[0].output.clone().into()
        );
        assert_eq!(live.data.as_deref(), Some(&[0x7e, 0x7f][..]));
        assert_eq!(live.block_hash, Some([8u8; 32]));
        let encoded = serde_molecule::to_vec(&live, false).unwrap();
        assert_eq!(
            serde_molecule::from_slice::<LiveCell>(&encoded, false).unwrap(),
            live
        );

        let without_data = live_cell(Some(&cells[0]), false);
        assert!(without_data.data.is_none());
        assert!(without_data.output.is_some());

        let unknown = live_cell(None, true);
        assert!(unknown.output.is_none());
        assert!(unknown.data.is_none());
        assert!(unknown.block_hash.is_none());
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

        let page = cells_page(&cells, &key, Order::Asc, 1, 0).unwrap();
        assert_eq!(page.objects.len(), 1);
        assert!(page.objects[0].output_data.is_none());
        assert_eq!(page.objects[0].block_number, 8);
        assert_eq!(page.objects[0].tx_index, 0);
        assert_eq!(page.objects[0].out_point, cells[0].out_point.clone().into());
        assert_eq!(page.objects[0].output, cells[0].output.clone().into());
        assert_eq!(page.last_cursor, 1u64.to_le_bytes());
        let encoded = serde_molecule::to_vec(&page, false).unwrap();
        assert_eq!(
            serde_molecule::from_slice::<Pagination>(&encoded, false).unwrap(),
            page
        );

        let next = cells_page(&cells, &key, Order::Asc, 1, 1).unwrap();
        assert_eq!(next.objects[0].out_point, cells[1].out_point.clone().into());
        assert_eq!(next.objects[0].block_number, 0);
        assert!(next.last_cursor.is_empty());

        let desc = cells_page(&cells, &key, Order::Desc, 1, 0).unwrap();
        assert_eq!(desc.objects[0].out_point, cells[1].out_point.clone().into());

        let empty = cells_page(&cells, &key, Order::Asc, 0, 0).unwrap();
        assert!(empty.objects.is_empty());
        assert_eq!(empty.last_cursor, 0u64.to_le_bytes());
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
        let exact_page = cells_page(&cells, &exact, Order::Asc, 10, 0).unwrap();
        assert_eq!(exact_page.objects.len(), 1);
        assert_eq!(
            exact_page.objects[0].output_data.as_deref(),
            Some(&[0x7e, 0x7f][..])
        );

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
        let partial_page = cells_page(&cells, &partial, Order::Asc, 10, 0).unwrap();
        assert_eq!(partial_page.objects.len(), 1);
        assert_eq!(
            partial_page.objects[0].out_point,
            cells[1].out_point.clone().into()
        );

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
        let type_page = cells_page(&cells, &type_key, Order::Asc, 10, 0).unwrap();
        assert_eq!(type_page.objects.len(), 1);
    }

    #[test]
    fn serde_molecule_matches_packed_bytes() {
        let cells = sample_cells();
        let packed_script = script(0x11, 0, &[0xaa, 0x01]);
        let script = Script::from(packed_script.clone());
        assert_eq!(
            serde_molecule::to_vec(&script, false).unwrap(),
            packed_script.as_slice()
        );

        let output = CellOutput::from(cells[0].output.clone());
        assert_eq!(
            serde_molecule::to_vec(&output, false).unwrap(),
            cells[0].output.as_slice()
        );

        let out_point = OutPoint::from(cells[0].out_point.clone());
        assert_eq!(
            serde_molecule::to_vec(&out_point, true).unwrap(),
            cells[0].out_point.as_slice()
        );
    }
}
