//! Permanent identities are supplied, never derived from an open-session ID.
use core::num::NonZeroU128;
use std::collections::BTreeMap;

use crate::NodeId;

macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
        pub struct $name(NonZeroU128);
        impl $name {
            pub const fn new(value: u128) -> Option<Self> {
                match NonZeroU128::new(value) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }
            pub const fn get(self) -> u128 {
                self.0.get()
            }
        }
    };
}
identity!(PersistentHistoryId);
identity!(PersistentNodeId);
identity!(LineageId);
identity!(AddressEpoch);
identity!(CommitId);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityError {
    Unavailable,
}

/// The host supplies fresh random candidates. The owner rejects zero/collisions;
/// consuming a candidate does not admit it or create a project operation.
pub trait HistoryIdentitySource {
    fn next_identity(&mut self) -> Result<u128, IdentityError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistentHistoryIdentity {
    pub history: PersistentHistoryId,
    pub lineage: LineageId,
    pub address_epoch: AddressEpoch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistentNodeReference {
    pub project: crate::values::ProjectId,
    pub history: PersistentHistoryId,
    pub node: PersistentNodeId,
}
impl PersistentNodeReference {
    pub fn text(self) -> String {
        format!(
            "hp1:{:032x}:{:032x}:{:032x}",
            self.project.get(),
            self.history.get(),
            self.node.get()
        )
    }
    /// Canonical syntax only; a future open-file port must also check scope.
    pub fn parse(text: &str) -> Option<Self> {
        if text.len() != 102 {
            return None;
        }
        let mut parts = text.split(':');
        if parts.next()? != "hp1" {
            return None;
        }
        let mut value = || {
            let part = parts.next()?;
            if part.len() != 32
                || !part
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return None;
            }
            u128::from_str_radix(part, 16).ok()
        };
        let result = Self {
            project: crate::values::ProjectId::new(value()?)?,
            history: PersistentHistoryId::new(value()?)?,
            node: PersistentNodeId::new(value()?)?,
        };
        if parts.next().is_some() {
            return None;
        }
        Some(result)
    }
}

#[derive(Clone)]
pub(crate) struct PersistenceMap {
    pub identity: PersistentHistoryIdentity,
    pub nodes: BTreeMap<NodeId, PersistentNodeId>,
}

impl PersistentNodeId {
    pub fn short_key(self) -> String {
        let mut n = self.get();
        let mut digits = [b'0'; 25];
        let mut start = digits.len();
        loop {
            start -= 1;
            let d = (n % 36) as u8;
            digits[start] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
            n /= 36;
            if n == 0 {
                break;
            }
        }
        let mut result = String::from("k");
        result.extend(digits[start..].iter().map(|b| char::from(*b)));
        result
    }
    pub fn parse_short_key(text: &str) -> Option<Self> {
        let digits = text.strip_prefix('k')?;
        if digits.is_empty()
            || digits.len() > 25
            || digits.starts_with('0')
            || !digits
                .bytes()
                .all(|b| b.is_ascii_digit() || b.is_ascii_lowercase())
        {
            return None;
        }
        Self::new(u128::from_str_radix(digits, 36).ok()?)
    }
}
