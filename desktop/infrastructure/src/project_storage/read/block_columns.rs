use super::*;
#[cfg(test)]
mod tests;

pub(super) fn verify(
    r: &rusqlite::Row<'_>,
    parsed: blocks::ParsedBlock,
) -> Result<blocks::Row, StorageError> {
    let blocks::ParsedBlock { row, project_id } = parsed;
    if text_column(r, 4, false)? != Some(row.kind.as_str()) || r.get::<_, i64>(5)? != 1 {
        return Err(StorageError::Invalid("block derived fields"));
    }
    let expected = project_id?;
    if text_column(r, 6, true)? != expected.as_deref() {
        return Err(StorageError::Invalid("block project"));
    }
    Ok(row)
}
