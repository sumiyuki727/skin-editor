use super::*;
#[cfg(test)]
mod tests;

pub(in crate::project_storage) const METADATA_ITEMS: usize = 8192;
type Order<const N: usize> = [u16; N];
// One role at a time. Include the index initializer/destination, comparison
// keys, digest values and fixed locals. The page DOM has separate owners.
pub(in crate::project_storage) const METADATA_WORK_BYTES: usize =
    2 * size_of::<Order<METADATA_ITEMS>>()
        + 4 * size_of::<(Option<&str>, Option<&str>, u16)>()
        + 2 * (size_of::<Sha256>() + size_of::<[u8; 32]>())
        + size_of::<[usize; 128]>();
const _: () = assert!(METADATA_ITEMS <= u16::MAX as usize);

struct Object<'a>(&'a serde_json::Map<String, Value>);
impl<'a> Object<'a> {
    fn new(v: &'a Value) -> Result<Self, StorageError> {
        v.as_object()
            .map(Self)
            .ok_or(StorageError::Invalid("object"))
    }
    fn get(&self, name: &'static str) -> Result<&'a Value, StorageError> {
        self.0.get(name).ok_or(StorageError::Invalid(name))
    }
    fn text(&self, name: &'static str) -> Result<&'a str, StorageError> {
        self.get(name)?
            .as_str()
            .ok_or(StorageError::Invalid("text"))
    }
    // Call after checking every required field, in format order.
    fn finish(&self) -> Result<(), StorageError> {
        if self.0.len() != 3 {
            return Err(StorageError::Unsupported("metadata field"));
        }
        Ok(())
    }
}

pub(super) fn bounded_feature(v: &Value) -> Result<Option<usize>, StorageError> {
    let f = Object::new(v)?;
    let name = f.text("name")?;
    let version = f.text("version")?;
    let required = f
        .get("required")?
        .as_bool()
        .ok_or(StorageError::Invalid("feature required"))?;
    f.finish()?;
    if name.is_empty() || name.len() > 256 || version.len() > 64 {
        return Err(StorageError::Limit("feature"));
    }
    if !required {
        return Ok(None);
    }
    write::REQUIRED_FEATURES
        .iter()
        .position(|&pair| pair == (name, version))
        .map(Some)
        .ok_or(StorageError::Unsupported("required feature"))
}

#[derive(Clone, Copy)]
enum Role {
    Tags,
    Extensions,
    Features,
}
impl Role {
    fn key(self, value: &Value) -> (Option<&str>, Option<&str>) {
        // Bound the pre-sort comparisons even on rows that will subsequently
        // fail field validation. Every accepted key fits these byte lengths.
        let text = |field, bytes| {
            value
                .get(field)
                .and_then(Value::as_str)
                .filter(|s| s.len() <= bytes)
        };
        let name = text("name", 256);
        match self {
            Self::Tags => (text("node", 32), name),
            Self::Extensions => (name, None),
            Self::Features => (name, text("version", 64)),
        }
    }

    // Only sort indices. The first repeated row in *source* order is all
    // the validator needs; malformed rows still get their original errors.
    fn first_duplicate<const N: usize>(self, rows: &[Value]) -> Option<usize> {
        let mut order: Order<N> = [0; N];
        let order = &mut order[..rows.len()];
        for (i, index) in order.iter_mut().enumerate() {
            *index = i as u16;
        }
        order.sort_unstable_by_key(|&i| (self.key(&rows[usize::from(i)]), i));
        order
            .windows(2)
            .filter_map(|w| {
                let first = usize::from(w[0]);
                let next = usize::from(w[1]);
                (self.key(&rows[first]) == self.key(&rows[next])).then_some(next)
            })
            .min()
    }

    fn validate(
        self,
        rows: &[Value],
        mut visit: impl FnMut(usize, &Value, bool) -> Result<(), StorageError>,
    ) -> Result<(), StorageError> {
        let first_duplicate = match rows.len() {
            0 | 1 => None,
            2..=128 => self.first_duplicate::<128>(rows),
            129..=METADATA_ITEMS => self.first_duplicate::<METADATA_ITEMS>(rows),
            // Standalone archive limits can be wider. Retain the original
            // incremental uniqueness check, borrowing the input strings.
            _ => {
                let mut seen = BTreeSet::new();
                for (i, row) in rows.iter().enumerate() {
                    visit(i, row, !seen.insert(self.key(row)))?;
                }
                return Ok(());
            }
        };
        for (i, row) in rows.iter().enumerate() {
            visit(i, row, Some(i) == first_duplicate)?;
        }
        Ok(())
    }
}

pub(super) fn validate_past_metadata(
    nodes: &BTreeSet<PersistentNodeId>,
    meta: &ArchiveMetadata,
    known_features: &[Value],
    tags: &[Value],
    extensions: &[Value],
    features: &[Value],
    compare_latest: bool,
) -> Result<bool, StorageError> {
    let mut same = compare_latest
        && tags.len() == meta.tags.len()
        && extensions.len() == meta.extensions.len()
        && features.len() == known_features.len();
    Role::Tags.validate(tags, |i, v, duplicate| {
        let f = Object::new(v)?;
        let node = pid(f.text("node")?)?;
        let name = f.text("name")?;
        let value = f.text("value")?;
        f.finish()?;
        if !nodes.contains(&node)
            || !name.starts_with("user:")
            || name.len() > 256
            || value.len() > 4096
            || duplicate
        {
            return Err(StorageError::Invalid("historical tags"));
        }
        if same {
            let t = &meta.tags[i];
            same = t.node == node && t.name == name && t.value == value;
        }
        Ok(())
    })?;
    Role::Extensions.validate(extensions, |i, v, duplicate| {
        let f = Object::new(v)?;
        let name = f.text("name")?;
        let digest = hash(f.text("sha")?)?.bytes();
        let len = decimal(f.text("bytes")?)?;
        f.finish()?;
        // metadata_rows reads the canonical table ORDER BY name. Its unique
        // name identifies the actual bytes, including unused extensions.
        let pos = meta
            .extensions
            .binary_search_by(|e| e.name.as_str().cmp(name))
            .map_err(|_| StorageError::Invalid("historical extension"))?;
        let ext = &meta.extensions[pos];
        if duplicate || sha(&ext.bytes) != digest || ext.bytes.len() as u64 != len {
            return Err(StorageError::Invalid("historical extension binding"));
        }
        same &= pos == i;
        Ok(())
    })?;
    let mut required_present = [false; write::REQUIRED_FEATURES.len()];
    Role::Features.validate(features, |i, v, duplicate| {
        if let Some(required) = bounded_feature(v)? {
            required_present[required] = true;
        }
        if !known_features.contains(v) || duplicate {
            return Err(StorageError::Invalid("historical features"));
        }
        same = same && known_features[i] == *v;
        Ok(())
    })?;
    if required_present.contains(&false) {
        return Err(StorageError::Unsupported("historical required feature"));
    }
    Ok(same)
}
