//! Raw-size admission ahead of Rust catalog/row copies; not a heap proof.
use super::*;

macro_rules! aggregate_sql {
    ($table:literal, $payload:literal) => {
        concat!(
            "SELECT count(*),coalesce(sum(x),0),coalesce(max(x),0),count(x),",
            "coalesce(sum(CASE WHEN typeof(x)!='integer' OR x<0 THEN 1 ELSE 0 END),0) ",
            "FROM (SELECT ",
            $payload,
            " AS x FROM ",
            $table,
            ")"
        )
    };
}
struct Plan {
    sql: &'static str,
    rows: u64,
    bytes: u64,
    largest: u64,
}
fn measure(
    c: &Connection,
    plan: Plan,
    cancel: &CaptureCancellation,
    limit: FileFactsLimit,
) -> Result<FileReadAggregate, FileFactsError> {
    if cancel.is_cancelled() {
        return Err(FileFactsError::Cancelled);
    }
    let (rows, bytes, largest, present, invalid) = c
        .query_row(plan.sql, [], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })
        .map_err(|e| {
            if cancel.is_cancelled() {
                FileFactsError::Cancelled
            } else {
                storage(e.into())
            }
        })?;
    if rows < 0 || bytes < 0 || largest < 0 || present != rows || invalid != 0 {
        return Err(FileFactsError::ValidationFailed);
    }
    let value = FileReadAggregate {
        rows: rows as u64,
        bytes: bytes as u64,
        largest: largest as u64,
    };
    if value.rows > plan.rows || value.bytes > plan.bytes || value.largest > plan.largest {
        return Err(FileFactsError::Limit(limit));
    }
    Ok(value)
}
pub(super) fn scan(
    c: &Connection,
    limits: FileFactsLimits,
    observation: FileObservationId,
    cancel: &CaptureCancellation,
) -> Result<FileReadPreflight, FileFactsError> {
    let result = scan_inner(c, limits, observation, cancel);
    if cancel.is_cancelled() && result.is_err() {
        Err(FileFactsError::Cancelled)
    } else {
        result
    }
}
fn scan_inner(
    c: &Connection,
    limits: FileFactsLimits,
    observation: FileObservationId,
    cancel: &CaptureCancellation,
) -> Result<FileReadPreflight, FileFactsError> {
    let encoding: String = c
        .query_row("PRAGMA encoding", [], |r| r.get(0))
        .map_err(StorageError::from)
        .map_err(storage)?;
    if encoding != "UTF-8" {
        return Err(FileFactsError::Unsupported);
    }
    let catalog = measure(
        c,
        Plan {
            sql: aggregate_sql!(
                "sqlite_schema",
                "length(CAST(type AS BLOB))+length(CAST(name AS BLOB))+length(CAST(tbl_name AS BLOB))+coalesce(length(CAST(sql AS BLOB)),0)"
            ),
            rows: 128,
            bytes: 128 * 1024,
            largest: 64 * 1024,
        },
        cancel,
        FileFactsLimit::Validation,
    )?;
    // Only the verified schema may supply the table queries below.
    schema::verify(c).map_err(storage)?;
    let meta = limits.metadata_bytes as u64;
    let nodes = limits.nodes as u64;
    let commits = limits.commits as u64;
    let raw = limits.original_bytes;
    let plans = [
        Plan {
            sql: aggregate_sql!(
                "header",
                "length(CAST(family AS BLOB))+length(CAST(profile AS BLOB))"
            ),
            rows: 1,
            bytes: 1024,
            largest: 1024,
        },
        Plan {
            sql: aggregate_sql!(
                "features",
                "length(CAST(name AS BLOB))+length(CAST(version AS BLOB))"
            ),
            rows: 256,
            bytes: meta,
            largest: 4096,
        },
        Plan {
            sql: aggregate_sql!("blobs", "byte_len"),
            rows: limits.originals as u64,
            bytes: raw,
            largest: raw.min(65536),
        },
        Plan {
            sql: aggregate_sql!("blob_chunks", "length(CAST(bytes AS BLOB))"),
            rows: limits.originals as u64,
            bytes: raw,
            largest: 65536,
        },
        Plan {
            sql: aggregate_sql!(
                "blocks",
                "length(CAST(bytes AS BLOB))+length(CAST(kind AS BLOB))+coalesce(length(CAST(project_id AS BLOB)),0)"
            ),
            rows: 8192,
            bytes: meta,
            largest: 256 * 1024 + 1024,
        },
        Plan {
            sql: aggregate_sql!(
                "block_refs",
                "length(CAST(owner AS BLOB))+length(CAST(target AS BLOB))"
            ),
            rows: 65536,
            bytes: 65536 * 64,
            largest: 64,
        },
        Plan {
            sql: aggregate_sql!(
                "blob_refs",
                "length(CAST(owner AS BLOB))+length(CAST(target AS BLOB))"
            ),
            rows: 65536,
            bytes: 65536 * 64,
            largest: 64,
        },
        Plan {
            sql: aggregate_sql!("nodes", "length(CAST(action AS BLOB))"),
            rows: nodes,
            bytes: meta,
            largest: 65536,
        },
        Plan {
            sql: aggregate_sql!(
                "addresses",
                "length(CAST(node_id AS BLOB))+length(CAST(alias AS BLOB))+length(CAST(branch AS BLOB))+length(CAST(ordinal AS BLOB))+length(CAST(checksum AS BLOB))"
            ),
            rows: nodes,
            bytes: nodes * 72,
            largest: 72,
        },
        Plan {
            sql: aggregate_sql!("state_views", "0"),
            rows: 0,
            bytes: 0,
            largest: 0,
        },
        Plan {
            sql: aggregate_sql!("commits", "length(CAST(origin AS BLOB))"),
            rows: commits,
            bytes: meta,
            largest: 32,
        },
        Plan {
            sql: aggregate_sql!(
                "manual_marks",
                "length(CAST(commit_id AS BLOB))+length(CAST(node_id AS BLOB))"
            ),
            rows: commits,
            bytes: commits * 32,
            largest: 32,
        },
        Plan {
            sql: aggregate_sql!("object_ids", "length(CAST(kind AS BLOB))"),
            rows: limits.admissions as u64,
            bytes: meta,
            largest: 32,
        },
        Plan {
            sql: aggregate_sql!(
                "tags",
                "length(CAST(name AS BLOB))+length(CAST(value AS BLOB))"
            ),
            rows: nodes * 64,
            bytes: meta,
            largest: 4352,
        },
        Plan {
            sql: aggregate_sql!(
                "control",
                "length(CAST(generation AS BLOB))+coalesce(length(CAST(save_id AS BLOB)),0)+coalesce(length(CAST(manual_id AS BLOB)),0)+coalesce(length(CAST(protected_id AS BLOB)),0)+length(CAST(address_epoch AS BLOB))"
            ),
            rows: 1,
            bytes: 72,
            largest: 72,
        },
        Plan {
            sql: aggregate_sql!(
                "extensions",
                "length(CAST(name AS BLOB))+length(CAST(bytes AS BLOB))"
            ),
            rows: 1024,
            bytes: meta,
            largest: meta,
        },
    ];
    let mut tables = [FileReadAggregate::default(); FILE_READ_PREFLIGHT_TABLES];
    for (i, plan) in plans.into_iter().enumerate() {
        tables[i] = measure(
            c,
            plan,
            cancel,
            if i == 10 {
                FileFactsLimit::Commits
            } else {
                FileFactsLimit::Validation
            },
        )?;
    }
    if cancel.is_cancelled() {
        return Err(FileFactsError::Cancelled);
    }
    Ok(FileReadPreflight {
        version: FILE_READ_PREFLIGHT_VERSION,
        observation,
        catalog,
        tables,
    })
}
