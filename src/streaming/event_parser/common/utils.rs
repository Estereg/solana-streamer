use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp in seconds
#[must_use]
pub fn current_timestamp() -> i64 {
    i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()).unwrap_or(i64::MAX)
}

/// Safely read a u64 from a byte array (little-endian)
#[must_use]
pub fn read_u64_le(data: &[u8], offset: usize) -> Option<u64> {
    if data.len() < offset + 8 {
        return None;
    }
    let bytes: [u8; 8] = data[offset..offset + 8].try_into().ok()?;
    Some(u64::from_le_bytes(bytes))
}

#[must_use]
pub fn read_i32_le(data: &[u8], offset: usize) -> Option<i32> {
    if data.len() < offset + 4 {
        return None;
    }
    let bytes: [u8; 4] = data[offset..offset + 4].try_into().ok()?;
    Some(i32::from_le_bytes(bytes))
}

#[must_use]
pub fn read_u128_le(data: &[u8], offset: usize) -> Option<u128> {
    if data.len() < offset + 16 {
        return None;
    }
    let bytes: [u8; 16] = data[offset..offset + 16].try_into().ok()?;
    Some(u128::from_le_bytes(bytes))
}

#[must_use]
pub fn read_option_bool(data: &[u8], offset: &mut usize) -> Option<Option<bool>> {
    let has_value = data.get(*offset).copied()?;
    *offset += 1;

    if has_value == 0 {
        return Some(None);
    }

    let value = data.get(*offset).copied()?;
    *offset += 1;

    Some(Some(value != 0))
}

/// Safely read a u32 from a byte array (little-endian)
#[must_use]
pub fn read_u32_le(data: &[u8], offset: usize) -> Option<u32> {
    if data.len() < offset + 4 {
        return None;
    }
    let bytes: [u8; 4] = data[offset..offset + 4].try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

/// Safely read a u16 from a byte array (little-endian)
#[must_use]
pub fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
    if data.len() < offset + 2 {
        return None;
    }
    let bytes: [u8; 2] = data[offset..offset + 2].try_into().ok()?;
    Some(u16::from_le_bytes(bytes))
}

/// Safely read a u8 from a byte array
#[must_use]
pub fn read_u8(data: &[u8], offset: usize) -> Option<u8> {
    data.get(offset).copied()
}
