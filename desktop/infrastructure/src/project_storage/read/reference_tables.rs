use super::*;
#[cfg(test)]
mod tests;

pub(super) fn verify(
    c: &Connection,
    blocks: &BTreeMap<StateHash, blocks::Row>,
    cancel: &CaptureCancellation,
) -> Result<(), StorageError> {
    let mut q = c.prepare("SELECT owner,target FROM block_refs ORDER BY owner,target")?;
    let mut rows = q.query([])?;
    for (owner, row) in blocks {
        for target in &row.refs {
            check(cancel)?;
            let r = rows
                .next()?
                .ok_or(StorageError::Invalid("missing block index"))?;
            if fixed_blob_column(r, 0)? != &owner.bytes()
                || fixed_blob_column(r, 1)? != &target.bytes()
            {
                return Err(StorageError::Invalid("block index"));
            }
        }
    }
    if rows.next()?.is_some() {
        return Err(StorageError::Invalid("extra block index"));
    }
    drop(rows);
    drop(q);
    let mut q = c.prepare("SELECT owner,target FROM blob_refs ORDER BY owner,target")?;
    let mut rows = q.query([])?;
    for (owner, row) in blocks {
        for target in &row.blobs {
            check(cancel)?;
            let r = rows
                .next()?
                .ok_or(StorageError::Invalid("missing blob index"))?;
            if fixed_blob_column(r, 0)? != &owner.bytes() || fixed_blob_column(r, 1)? != target {
                return Err(StorageError::Invalid("blob index"));
            }
        }
    }
    if rows.next()?.is_some() {
        return Err(StorageError::Invalid("extra blob index"));
    }
    Ok(())
}
