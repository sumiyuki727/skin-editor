use super::*;

const INLINE_PAGES: usize = read::METADATA_ITEMS.div_ceil(PAGE_ITEMS);

/// Owns the parser's original item buffers in index order. Every page except
/// the last is full, as checked by visit_pages before handing it over.
pub(in crate::project_storage) struct MetadataPages {
    pages: [Vec<Value>; INLINE_PAGES],
    // Only standalone archives with more rows than Small need additional slots.
    overflow: Vec<Vec<Value>>,
    len: usize,
}

impl MetadataPages {
    fn empty() -> Self {
        Self {
            pages: [const { Vec::new() }; INLINE_PAGES],
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
        let read_page = if limits.codec.items <= read::METADATA_ITEMS {
            compact_page_value
        } else {
            value
        };
        visit_pages(cache, root, role, limits, read_page, |page| rows.push(page))?;
        Ok(rows)
    }

    fn push(&mut self, page: Vec<Value>) {
        let index = self.len / PAGE_ITEMS;
        self.len += page.len();
        if let Some(slot) = self.pages.get_mut(index) {
            *slot = page;
        } else {
            self.overflow.push(page);
        }
    }

    pub(in crate::project_storage) fn len(&self) -> usize {
        self.len
    }

    pub(in crate::project_storage) fn iter(&self) -> impl Iterator<Item = &Value> {
        self.pages[..self.len.div_ceil(PAGE_ITEMS).min(INLINE_PAGES)]
            .iter()
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
        let page = index / PAGE_ITEMS;
        let rows = self
            .pages
            .get(page)
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
        }
    }
}
