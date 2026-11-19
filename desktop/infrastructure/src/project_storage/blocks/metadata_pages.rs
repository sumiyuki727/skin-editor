use super::*;

const INLINE_PAGES: usize = read::METADATA_ITEMS.div_ceil(PAGE_ITEMS);
const _: () = assert!(INLINE_PAGES <= u8::MAX as usize + 1);

/// A Small index can name the same immutable page at multiple positions.
/// The first occurrence owns its validated buffer. Later positions refer to it.
pub(in crate::project_storage) struct MetadataPages {
    pages: [Vec<Value>; INLINE_PAGES],
    owners: [u8; INLINE_PAGES],
    // Only standalone archives with more rows than Small need additional slots.
    overflow: Vec<Vec<Value>>,
    len: usize,
}

impl MetadataPages {
    fn empty() -> Self {
        Self {
            pages: [const { Vec::new() }; INLINE_PAGES],
            owners: [0; INLINE_PAGES],
            overflow: Vec::new(),
            len: 0,
        }
    }

    pub(in crate::project_storage) fn read(
        cache: &Cache,
        root: StateHash,
        role: &str,
        limits: FileLimits,
    ) -> Result<Self, StorageError> {
        let mut rows = Self::empty();
        let index = page_index(cache, root, role, limits)?;
        let small = limits.codec.items <= read::METADATA_ITEMS;
        let read_page = if small { compact_page_value } else { value };
        for (i, (hash, count)) in index.entries().enumerate() {
            if small && let Some(first) = index.pages[..i].iter().position(|h| *h == hash) {
                // All checks except the position-dependent tail length were
                // made on this very page, role and immutable cache.
                if rows.pages[first].len() != count {
                    return Err(StorageError::Invalid("page count"));
                }
                rows.owners[i] = first as u8;
                rows.len += count;
            } else {
                rows.push(read_page_items(
                    cache,
                    hash,
                    role,
                    count,
                    limits.codec,
                    read_page,
                )?);
            }
        }
        Ok(rows)
    }

    fn push(&mut self, page: Vec<Value>) {
        let index = self.len / PAGE_ITEMS;
        self.len += page.len();
        if let Some(slot) = self.pages.get_mut(index) {
            *slot = page;
            self.owners[index] = index as u8;
        } else {
            self.overflow.push(page);
        }
    }

    pub(in crate::project_storage) fn len(&self) -> usize {
        self.len
    }

    pub(in crate::project_storage) fn iter(&self) -> impl Iterator<Item = &Value> {
        self.owners[..self.len.div_ceil(PAGE_ITEMS).min(INLINE_PAGES)]
            .iter()
            .map(|&slot| &self.pages[usize::from(slot)])
            .chain(&self.overflow)
            .flatten()
    }

    #[cfg(test)]
    pub(in crate::project_storage) fn from_rows(rows: &[Value]) -> Self {
        let mut out = Self::empty();
        for page in rows.chunks(PAGE_ITEMS) {
            out.push(page.to_vec());
        }
        out
    }
}

impl std::ops::Index<usize> for MetadataPages {
    type Output = Value;

    fn index(&self, index: usize) -> &Value {
        assert!(index < self.len);
        let page = index / PAGE_ITEMS;
        let rows = self
            .owners
            .get(page)
            .map(|&slot| &self.pages[usize::from(slot)])
            .unwrap_or_else(|| &self.overflow[page - INLINE_PAGES]);
        &rows[index % PAGE_ITEMS]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validated_page_transfers_the_original_buffer_including_spare_capacity() {
        for count in [
            0,
            1,
            255,
            256,
            257,
            read::METADATA_ITEMS,
            read::METADATA_ITEMS + 1,
        ] {
            let mut rows = MetadataPages::empty();
            for start in (0..count).step_by(PAGE_ITEMS) {
                let end = (start + PAGE_ITEMS).min(count);
                let mut page = Vec::with_capacity(PAGE_ITEMS + 17);
                page.extend((start..end).map(Value::from));
                let (ptr, capacity) = (page.as_ptr(), page.capacity());
                rows.push(page);
                let held = rows
                    .pages
                    .iter()
                    .chain(&rows.overflow)
                    .nth(start / PAGE_ITEMS)
                    .unwrap();
                assert_eq!(held.as_ptr(), ptr);
                assert_eq!(held.capacity(), capacity);
            }
            assert_eq!(rows.len(), count);
            if count <= read::METADATA_ITEMS {
                assert_eq!(rows.overflow.capacity(), 0);
            }
            for (i, row) in rows.iter().enumerate() {
                assert_eq!(row.as_u64(), Some(i as u64));
                assert!(std::ptr::eq(row, &rows[i]));
            }
            assert_eq!(rows.iter().count(), count);
            assert!(std::panic::catch_unwind(|| &rows[count]).is_err());
        }
    }
}
