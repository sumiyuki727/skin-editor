use core::ops::Range;

use crate::ModelError;

/// Interpretation indexes bytes in a preserved record, without normalising text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigInterpretation {
    Opaque,
    Section {
        name: Range<usize>,
    },
    Entry {
        key: Range<usize>,
        value: Range<usize>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigRecord {
    raw: Vec<u8>,
    interpretation: ConfigInterpretation,
}

impl ConfigRecord {
    /// Does not parse skin.ini or claim that an annotated key is supported.
    pub fn new(raw: Vec<u8>, interpretation: ConfigInterpretation) -> Result<Self, ModelError> {
        let valid = |range: &Range<usize>, allow_empty| {
            range.start <= range.end
                && range.end <= raw.len()
                && (allow_empty || range.start < range.end)
        };
        let accepted = match &interpretation {
            ConfigInterpretation::Opaque => true,
            ConfigInterpretation::Section { name } => valid(name, false),
            ConfigInterpretation::Entry { key, value } => {
                valid(key, false) && valid(value, true) && key.end <= value.start
            }
        };
        if !accepted {
            return Err(ModelError::InvalidValue("configuration_range"));
        }
        Ok(Self {
            raw,
            interpretation,
        })
    }

    pub fn raw(&self) -> &[u8] {
        &self.raw
    }

    pub fn interpretation(&self) -> &ConfigInterpretation {
        &self.interpretation
    }
}

/// Ordered lossless records, including comments, duplicate/unknown entries,
/// mixed line endings and bytes whose encoding is not yet interpreted.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SkinConfiguration {
    records: Vec<ConfigRecord>,
}

impl SkinConfiguration {
    pub const fn new(records: Vec<ConfigRecord>) -> Self {
        Self { records }
    }

    pub fn records(&self) -> &[ConfigRecord] {
        &self.records
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.records
            .iter()
            .flat_map(|record| record.raw.iter().copied())
            .collect()
    }
}
