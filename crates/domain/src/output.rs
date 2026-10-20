use crate::{DocumentId, ModelError, OutputId};

/// A lexical relative output path. Filesystem and Unicode-normalisation checks
/// still belong to the eventual export adapter; this is not a stable name rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputPath(String);

impl OutputPath {
    pub fn new(path: String) -> Result<Self, ModelError> {
        if path.is_empty()
            || path
                .chars()
                .any(|c| c.is_control() || c == '\\' || c == ':')
            || path.split('/').any(|part| {
                part.is_empty()
                    || part == "."
                    || part == ".."
                    || part.ends_with('.')
                    || part.ends_with(' ')
                    || part
                        .chars()
                        .any(|c| matches!(c, '<' | '>' | '"' | '|' | '?' | '*'))
            })
        {
            return Err(ModelError::InvalidOutputPath(path));
        }
        Ok(Self(path))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn collision_key(&self) -> String {
        self.0.to_lowercase()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputMapping {
    id: OutputId,
    document: DocumentId,
    element_key: String,
    path: OutputPath,
}

impl OutputMapping {
    pub fn new(
        id: OutputId,
        document: DocumentId,
        element_key: String,
        path: OutputPath,
    ) -> Result<Self, ModelError> {
        if element_key.trim().is_empty() {
            return Err(ModelError::InvalidValue("element_key"));
        }
        Ok(Self {
            id,
            document,
            element_key,
            path,
        })
    }

    pub const fn id(&self) -> OutputId {
        self.id
    }

    pub const fn document(&self) -> DocumentId {
        self.document
    }

    pub fn element_key(&self) -> &str {
        &self.element_key
    }

    pub fn path(&self) -> &OutputPath {
        &self.path
    }
}
