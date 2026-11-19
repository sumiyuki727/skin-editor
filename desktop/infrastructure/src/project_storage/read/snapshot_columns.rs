use super::*;
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};

// SQL type errors belong to the row read. A wrong width is reported only
// when validation reaches the field, even if the row has long since gone.
struct DeferredBytes<const N: usize>(Option<[u8; N]>);

impl<const N: usize> FromSql for DeferredBytes<N> {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Ok(Self(value.as_blob()?.try_into().ok()))
    }
}

impl<const N: usize> DeferredBytes<N> {
    fn bytes(&self) -> Result<[u8; N], StorageError> {
        self.0.ok_or(StorageError::Invalid("fixed bytes"))
    }
}

impl DeferredBytes<16> {
    fn number(&self) -> Result<u128, StorageError> {
        Ok(u128::from_be_bytes(self.bytes()?))
    }

    fn commit(&self) -> Result<CommitId, StorageError> {
        CommitId::new(self.number()?).ok_or(StorageError::Invalid("commit id"))
    }
}

struct HeaderIds {
    project: DeferredBytes<16>,
    history: DeferredBytes<16>,
    lineage: DeferredBytes<16>,
    root: DeferredBytes<16>,
}

pub(super) struct Control {
    generation: DeferredBytes<8>,
    save: DeferredBytes<16>,
    manual: Option<DeferredBytes<16>>,
    protected: DeferredBytes<16>,
    pub(super) node_count: i64,
    pub(super) block_count: i64,
    pub(super) blob_bytes: i64,
    pub(super) metadata_bytes: i64,
}

impl Control {
    pub(super) fn matches_commits(
        &self,
        generation: u64,
        commit: CommitId,
        manual: Option<CommitId>,
    ) -> Result<bool, StorageError> {
        Ok(u64::from_be_bytes(self.generation.bytes()?) == generation
            && self.save.commit()? == commit
            && self
                .manual
                .as_ref()
                .map(DeferredBytes::commit)
                .transpose()?
                == manual
            && self.protected.commit()? == commit)
    }
}

fn header_ids(row: &rusqlite::Row<'_>) -> Result<HeaderIds, StorageError> {
    let family = text_column(row, 0, false)? == Some("skin-editor-project");
    let profile = text_column(row, 1, false)? == Some(FILE_PROFILE);
    let container_version = row.get::<_, i64>(2)?;
    let project_schema = row.get::<_, i64>(3)?;
    let history_schema = row.get::<_, i64>(4)?;
    let ids = HeaderIds {
        project: row.get(5)?,
        history: row.get(6)?,
        lineage: row.get(7)?,
        root: row.get(8)?,
    };
    if !family || !profile || (container_version, project_schema, history_schema) != (2, 1, 2) {
        return Err(StorageError::Unsupported("profile/schema"));
    }
    Ok(ids)
}

pub(super) fn read(c: &Connection) -> Result<(Header, Control), StorageError> {
    let ids = c.query_row(
        "SELECT family,profile,container_version,project_schema,history_schema,
                project_id,history_id,lineage_id,root_id FROM header",
        [],
        |row| Ok(header_ids(row)),
    )??;
    let (control, epoch) = c.query_row(
        "SELECT generation,save_id,manual_id,protected_id,node_count,block_count,
                blob_bytes,metadata_bytes,address_epoch FROM control",
        [],
        |r| {
            Ok((
                Control {
                    generation: r.get(0)?,
                    save: r.get(1)?,
                    manual: r.get(2)?,
                    protected: r.get(3)?,
                    node_count: r.get(4)?,
                    block_count: r.get(5)?,
                    blob_bytes: r.get(6)?,
                    metadata_bytes: r.get(7)?,
                },
                r.get::<_, DeferredBytes<16>>(8)?,
            ))
        },
    )?;
    let identity = PersistentHistoryIdentity {
        history: PersistentHistoryId::new(ids.history.number()?)
            .ok_or(StorageError::Invalid("history id"))?,
        lineage: LineageId::new(ids.lineage.number()?).ok_or(StorageError::Invalid("lineage"))?,
        address_epoch: AddressEpoch::new(epoch.number()?)
            .ok_or(StorageError::Invalid("address epoch"))?,
    };
    let header = Header {
        project: ProjectId::new(ids.project.number()?).ok_or(StorageError::Invalid("project"))?,
        identity,
        root: PersistentNodeId::new(ids.root.number()?).ok_or(StorageError::Invalid("node id"))?,
    };
    Ok((header, control))
}

#[cfg(test)]
mod tests;
