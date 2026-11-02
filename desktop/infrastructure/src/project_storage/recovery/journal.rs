//! Preflight only. SQLite performs rollback on the private byte copy.
use super::*;
use std::io::{Read, Seek, SeekFrom};
const MAGIC: [u8; 8] = [0xd9, 0xd5, 0x05, 0xf9, 0x20, 0xa1, 0x63, 0xd7];
fn number(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(bytes.try_into().expect("four bytes"))
}
fn zeros(
    file: &mut std::fs::File,
    start: u64,
    end: u64,
    cancel: &CaptureCancellation,
) -> Result<(), StorageError> {
    file.seek(SeekFrom::Start(start)).map_err(io)?;
    let mut pos = start;
    let mut buf = [0u8; 65536];
    while pos < end {
        check(cancel)?;
        let n = (end - pos).min(buf.len() as u64) as usize;
        file.read_exact(&mut buf[..n]).map_err(io)?;
        if buf[..n].iter().any(|b| *b != 0) {
            return Err(StorageError::Invalid("journal padding or tail"));
        }
        pos += n as u64;
    }
    Ok(())
}
pub(super) fn validate(
    source: &mut source::Frozen,
    limits: FileLimits,
    cancel: &CaptureCancellation,
) -> Result<u64, StorageError> {
    let Some(j) = &mut source.journal else {
        return Ok(0);
    };
    let len = j.digest.byte_len;
    if len == 0 {
        return Ok(0);
    }
    if len <= 512 {
        return Err(StorageError::Unsupported("non-hot or incomplete journal"));
    }
    // Reject a possible super-journal footer before SQLite could read or
    // delete any file named by it. False positives are deliberately refused.
    j.file.seek(SeekFrom::End(-8)).map_err(io)?;
    let mut tail = [0; 8];
    j.file.read_exact(&mut tail).map_err(io)?;
    if tail == MAGIC {
        return Err(StorageError::Unsupported("super-journal recovery"));
    }
    let mut position = 0u64;
    let mut sector = None;
    let mut original = None;
    let mut seen = BTreeSet::new();
    let max_records = (limits.file_bytes / 4096).min((limits.metadata_bytes / 32) as u64);
    while position < len {
        check(cancel)?;
        if len - position < 28 {
            return Err(StorageError::Invalid("journal header"));
        }
        j.file.seek(SeekFrom::Start(position)).map_err(io)?;
        let mut header = [0u8; 28];
        j.file.read_exact(&mut header).map_err(io)?;
        if position > 0
            && header[..12].iter().all(|b| *b == 0)
            && let Some(block) = sector
            && position + block == len
        {
            let zero = header.iter().all(|b| *b == 0);
            let pending = number(&header[24..28]) == 4096
                && u64::from(number(&header[20..24])) == block
                && original == Some(u64::from(number(&header[16..20])));
            if zero || pending {
                zeros(&mut j.file, position + 28, len, cancel)?;
                break;
            }
        }
        if header[..8] != MAGIC {
            return Err(StorageError::Unsupported("journal header"));
        }
        let mut count = u64::from(number(&header[8..12]));
        let nonce = number(&header[12..16]);
        let pages = u64::from(number(&header[16..20]));
        let block = u64::from(number(&header[20..24]));
        if number(&header[24..28]) != 4096
            || !(32..=65536).contains(&block)
            || !block.is_power_of_two()
            || pages == 0
            || pages > limits.file_bytes / 4096
            || pages > source.main.digest.byte_len / 4096
            || sector.is_some_and(|v| v != block)
            || original.is_some_and(|v| v != pages)
        {
            return Err(StorageError::Invalid("journal dimensions"));
        }
        sector = Some(block);
        original = Some(pages);
        let start = position
            .checked_add(block)
            .ok_or(StorageError::Limit("journal offset"))?;
        if start > len {
            return Err(StorageError::Invalid("journal header padding"));
        }
        zeros(&mut j.file, position + 28, start, cancel)?;
        if count == u32::MAX as u64 {
            if position != 0 || !(len - start).is_multiple_of(4104) {
                return Err(StorageError::Unsupported("unbounded journal tail"));
            }
            count = (len - start) / 4104;
        }
        if count == 0 {
            return Err(StorageError::Invalid("journal record count"));
        }
        if count > max_records.saturating_sub(seen.len() as u64) {
            return Err(StorageError::Limit("journal record metadata"));
        }
        let end = start
            .checked_add(
                count
                    .checked_mul(4104)
                    .ok_or(StorageError::Limit("journal records"))?,
            )
            .ok_or(StorageError::Limit("journal records"))?;
        if end > len {
            return Err(StorageError::Invalid("truncated journal records"));
        }
        j.file.seek(SeekFrom::Start(start)).map_err(io)?;
        for _ in 0..count {
            check(cancel)?;
            let mut pg = [0; 4];
            j.file.read_exact(&mut pg).map_err(io)?;
            let pg = number(&pg);
            if pg == 0 || pg == 0x40000000 / 4096 + 1 || u64::from(pg) > pages || !seen.insert(pg) {
                return Err(StorageError::Invalid("journal page number"));
            }
            let mut data = [0; 4096];
            j.file.read_exact(&mut data).map_err(io)?;
            let mut sum = nonce;
            let mut index = 4096usize;
            while let Some(next) = index.checked_sub(200) {
                index = next;
                sum = sum.wrapping_add(u32::from(data[index]));
            }
            let mut expected = [0; 4];
            j.file.read_exact(&mut expected).map_err(io)?;
            if sum != number(&expected) {
                return Err(StorageError::Invalid("journal checksum"));
            }
        }
        if end == len {
            break;
        }
        let next = end
            .checked_add(block - 1)
            .ok_or(StorageError::Limit("journal offset"))?
            / block
            * block;
        zeros(&mut j.file, end, next.min(len), cancel)?;
        if next >= len {
            break;
        }
        position = next;
    }
    Ok(seen.len() as u64)
}
