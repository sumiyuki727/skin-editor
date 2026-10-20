use std::collections::BTreeMap;

use crate::{ModelError, ObjectId};

/// Internal relationship validation shared by the two concrete tree models.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ForestIndex<I> {
    pub parents: BTreeMap<I, Option<I>>,
    pub preorder: Vec<I>,
}

impl<I> Default for ForestIndex<I> {
    fn default() -> Self {
        Self {
            parents: BTreeMap::new(),
            preorder: Vec::new(),
        }
    }
}

pub(crate) fn index_forest<I: Copy + Ord>(
    roots: &[I],
    edges: &BTreeMap<I, Vec<I>>,
    object: fn(I) -> ObjectId,
) -> Result<ForestIndex<I>, ModelError> {
    let mut parents = BTreeMap::new();
    for root in roots {
        if !edges.contains_key(root) {
            return Err(ModelError::MissingReference(object(*root)));
        }
        if parents.insert(*root, None).is_some() {
            return Err(ModelError::MultipleParents(object(*root)));
        }
    }
    for (parent, children) in edges {
        for child in children {
            if !edges.contains_key(child) {
                return Err(ModelError::MissingReference(object(*child)));
            }
            if parents.insert(*child, Some(*parent)).is_some() {
                return Err(ModelError::MultipleParents(object(*child)));
            }
        }
    }
    for id in edges.keys() {
        if !parents.contains_key(id) {
            return Err(ModelError::Unreachable(object(*id)));
        }
    }
    let mut stack: Vec<_> = roots.iter().rev().copied().collect();
    let mut preorder = Vec::with_capacity(edges.len());
    while let Some(id) = stack.pop() {
        preorder.push(id);
        stack.extend(edges[&id].iter().rev().copied());
    }
    // A cycle cannot be reachable from a root after unique-parent validation.
    if preorder.len() != edges.len() {
        return Err(ModelError::Cycle);
    }
    Ok(ForestIndex { parents, preorder })
}
