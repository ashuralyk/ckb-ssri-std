use alloc::vec::Vec;

pub fn encode_u64_vector(val: impl AsRef<[u64]>) -> Vec<u8> {
    let val = val.as_ref();
    u32::to_le_bytes(val.len() as u32)
        .into_iter()
        .chain(val.iter().flat_map(|v| u64::to_le_bytes(*v)))
        .collect()
}

#[allow(clippy::result_unit_err)]
pub fn decode_u64_vector(raw: impl AsRef<[u8]>) -> Result<Vec<u64>, ()> {
    let raw = raw.as_ref();

    Ok(raw
        .chunks(8)
        .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
        .collect())
}

pub fn encode_u8_32_vector(val: impl AsRef<[[u8; 32]]>) -> Vec<u8> {
    let val = val.as_ref();
    u32::to_le_bytes(val.len() as u32)
        .into_iter()
        .chain(val.iter().flat_map(|v| v.iter().copied()))
        .collect()
}

#[allow(clippy::result_unit_err)]
pub fn decode_u8_32_vector(raw: impl AsRef<[u8]>) -> Result<Vec<[u8; 32]>, ()> {
    let raw = raw.as_ref();
    let len = u32::from_le_bytes(raw[0..4].try_into().unwrap()) as usize;
    if len * 32 + 4 != raw.len() {
        return Err(());
    }

    Ok(raw[4..]
        .chunks(32)
        .map(|chunk| {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(chunk);
            arr
        })
        .collect())
}
