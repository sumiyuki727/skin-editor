use crate::{
    API_VERSION, AddressError, BranchCoordinate, HistoryAncestorsPage, HistoryChildrenPage,
    HistoryQueryError, HistorySessionId, HistoryStatistics, HistorySummary, MAX_BRANCH_ADDRESS,
    MAX_HISTORY_PAGE, NodeId, ProjectSession,
};

fn page_limit(limit: usize) -> Result<(), HistoryQueryError> {
    if limit == 0 || limit > MAX_HISTORY_PAGE {
        return Err(HistoryQueryError::InvalidPageLimit(limit));
    }
    Ok(())
}

impl ProjectSession {
    fn history_scope(&self, session_id: HistorySessionId) -> Result<(), HistoryQueryError> {
        if session_id != self.session_id {
            return Err(HistoryQueryError::WrongSession);
        }
        Ok(())
    }

    pub fn history_summary(
        &self,
        session_id: HistorySessionId,
        id: NodeId,
    ) -> Result<HistorySummary, HistoryQueryError> {
        self.history_scope(session_id)?;
        self.history.summary(id)
    }

    pub fn resolve_history_address(
        &self,
        session_id: HistorySessionId,
        address: &str,
    ) -> Result<NodeId, HistoryQueryError> {
        self.history_scope(session_id)?;
        if address.len() > MAX_BRANCH_ADDRESS {
            return Err(HistoryQueryError::InvalidAddress(AddressError::TooLong));
        }
        if address.starts_with('n') {
            let id = NodeId::parse_address(address).map_err(HistoryQueryError::InvalidAddress)?;
            self.history.node(id)?;
            Ok(id)
        } else {
            let coordinate = BranchCoordinate::parse_address(address)
                .map_err(HistoryQueryError::InvalidAddress)?;
            self.history
                .coordinates
                .get(&coordinate)
                .copied()
                .ok_or(HistoryQueryError::UnknownCoordinate(coordinate))
        }
    }

    /// Stable append order, with at most MAX_HISTORY_PAGE summaries per call.
    pub fn history_children(
        &self,
        session_id: HistorySessionId,
        parent: NodeId,
        offset: usize,
        limit: usize,
    ) -> Result<HistoryChildrenPage, HistoryQueryError> {
        self.history_scope(session_id)?;
        page_limit(limit)?;
        let children = &self.history.node(parent)?.children;
        if offset > children.len() {
            return Err(HistoryQueryError::InvalidOffset {
                offset,
                len: children.len(),
            });
        }
        let end = offset.saturating_add(limit).min(children.len());
        let entries = children[offset..end]
            .iter()
            .map(|id| self.history.summary(*id))
            .collect::<Result<_, _>>()?;
        Ok(HistoryChildrenPage {
            api_version: API_VERSION,
            session_id,
            parent,
            entries,
            next_offset: (end < children.len()).then_some(end),
        })
    }

    /// Starts with `start`; next_ancestor resumes without duplicating entries.
    pub fn history_ancestors(
        &self,
        session_id: HistorySessionId,
        start: NodeId,
        limit: usize,
    ) -> Result<HistoryAncestorsPage, HistoryQueryError> {
        self.history_scope(session_id)?;
        page_limit(limit)?;
        let mut cursor = Some(start);
        let mut entries = Vec::new();
        while entries.len() < limit {
            let Some(id) = cursor else {
                break;
            };
            let summary = self.history.summary(id)?;
            cursor = summary.parent;
            entries.push(summary);
        }
        Ok(HistoryAncestorsPage {
            api_version: API_VERSION,
            session_id,
            entries,
            next_ancestor: cursor,
        })
    }

    pub fn history_statistics(
        &self,
        session_id: HistorySessionId,
    ) -> Result<HistoryStatistics, HistoryQueryError> {
        self.history_scope(session_id)?;
        Ok(HistoryStatistics {
            api_version: API_VERSION,
            session_id,
            nodes: self.history.nodes.len(),
            edges: self.history.nodes.len().saturating_sub(1),
            coordinate_entries: self.history.coordinates.len(),
            branches: self.history.branch_count,
            max_depth: self.history.max_depth,
            retained_blobs: self.pins.len(),
            retained_requests: self.receipts.len(),
        })
    }
}
