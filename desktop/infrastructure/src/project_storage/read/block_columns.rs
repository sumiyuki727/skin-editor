use super::*;
use rusqlite::types::ValueRef;
#[cfg(test)]
mod tests;

// Use the same type and UTF-8 errors as Row::get<String>, without copying
// potentially long column values. Only project_id accepts SQL NULL.
fn text_column<'a>(
    r: &'a rusqlite::Row<'_>,
    index: usize,
    nullable: bool,
) -> Result<Option<&'a str>, StorageError> {
    match r.get_ref(index)? {
        ValueRef::Text(bytes) => Ok(Some(
            std::str::from_utf8(bytes).map_err(|e| rusqlite::Error::Utf8Error(index, e))?,
        )),
        ValueRef::Null if nullable => Ok(None),
        _ => Err(StorageError::Sql {
            code: None,
            class: "column type",
        }),
    }
}

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
