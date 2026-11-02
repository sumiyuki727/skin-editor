"""Private prototype SQLite schema. No arbitrary write port is exposed."""

TABLES = """
CREATE TABLE header (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 family TEXT NOT NULL, profile TEXT NOT NULL,
 container_version INTEGER NOT NULL, project_schema INTEGER NOT NULL, history_schema INTEGER NOT NULL,
 project_id BLOB NOT NULL CHECK(length(project_id)=16),
 history_id BLOB NOT NULL CHECK(length(history_id)=16),
 lineage_id BLOB NOT NULL CHECK(length(lineage_id)=16),
 root_id BLOB NOT NULL CHECK(length(root_id)=16)
);
CREATE TABLE features (name TEXT NOT NULL, version TEXT NOT NULL, required INTEGER NOT NULL CHECK(required IN (0,1)), PRIMARY KEY(name,version));
CREATE TABLE blobs (sha BLOB PRIMARY KEY CHECK(length(sha)=32), byte_len INTEGER NOT NULL CHECK(byte_len>=0));
CREATE TABLE blob_chunks (sha BLOB NOT NULL REFERENCES blobs(sha), part INTEGER NOT NULL CHECK(part>=0), bytes BLOB NOT NULL CHECK(length(bytes)>0 AND length(bytes)<=65536), PRIMARY KEY(sha,part));
CREATE TABLE blocks (sha BLOB PRIMARY KEY CHECK(length(sha)=32), sequence BLOB NOT NULL UNIQUE CHECK(length(sequence)=8), bytes BLOB NOT NULL, kind TEXT NOT NULL, schema_version INTEGER NOT NULL, project_id TEXT);
CREATE TABLE block_refs (owner BLOB NOT NULL REFERENCES blocks(sha), target BLOB NOT NULL REFERENCES blocks(sha), PRIMARY KEY(owner,target));
CREATE TABLE blob_refs (owner BLOB NOT NULL REFERENCES blocks(sha), target BLOB NOT NULL REFERENCES blobs(sha), PRIMARY KEY(owner,target));
CREATE TABLE nodes (
 id BLOB PRIMARY KEY CHECK(length(id)=16), parent BLOB REFERENCES nodes(id),
 state BLOB NOT NULL REFERENCES blocks(sha), depth INTEGER NOT NULL CHECK(depth>=0),
 action BLOB NOT NULL, checksum BLOB NOT NULL CHECK(length(checksum)=32),
 record_version INTEGER NOT NULL CHECK(record_version=2)
);
CREATE UNIQUE INDEX one_history_root ON nodes((1)) WHERE parent IS NULL;
CREATE INDEX node_children ON nodes(parent,id);
CREATE TABLE addresses (
 node_id BLOB PRIMARY KEY REFERENCES nodes(id), alias BLOB NOT NULL UNIQUE CHECK(length(alias)=8),
 branch BLOB NOT NULL CHECK(length(branch)=8), ordinal BLOB NOT NULL CHECK(length(ordinal)=8),
 checksum BLOB NOT NULL CHECK(length(checksum)=32), UNIQUE(branch,ordinal)
);
CREATE TABLE state_views (
 origin BLOB NOT NULL REFERENCES blocks(sha), target_schema INTEGER NOT NULL CHECK(target_schema=2),
 target BLOB NOT NULL REFERENCES blocks(sha), converter TEXT NOT NULL,
 checksum BLOB NOT NULL CHECK(length(checksum)=32), PRIMARY KEY(origin,target_schema,converter)
);
CREATE TABLE commits (
 id BLOB PRIMARY KEY CHECK(length(id)=16), previous BLOB REFERENCES commits(id),
 generation BLOB NOT NULL UNIQUE CHECK(length(generation)=8), origin TEXT NOT NULL,
 head BLOB REFERENCES nodes(id), navigation BLOB REFERENCES blocks(sha),
 request_hash BLOB NOT NULL CHECK(length(request_hash)=32), address_epoch BLOB NOT NULL CHECK(length(address_epoch)=16)
);
CREATE TABLE manual_marks (commit_id BLOB PRIMARY KEY REFERENCES commits(id), node_id BLOB NOT NULL REFERENCES nodes(id));
CREATE TABLE object_ids (kind TEXT NOT NULL, id BLOB NOT NULL CHECK(length(id)=16), first_node BLOB NOT NULL REFERENCES nodes(id), PRIMARY KEY(kind,id));
CREATE TABLE tags (node_id BLOB NOT NULL REFERENCES nodes(id), name TEXT NOT NULL, value TEXT NOT NULL, PRIMARY KEY(node_id,name));
CREATE TABLE control (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1), generation BLOB NOT NULL CHECK(length(generation)=8),
 save_id BLOB REFERENCES commits(id), manual_id BLOB REFERENCES commits(id), protected_id BLOB REFERENCES commits(id),
 node_count INTEGER NOT NULL, block_count INTEGER NOT NULL, blob_bytes INTEGER NOT NULL, metadata_bytes INTEGER NOT NULL,
 address_epoch BLOB NOT NULL CHECK(length(address_epoch)=16)
);
CREATE TABLE extensions (name TEXT PRIMARY KEY, bytes BLOB NOT NULL);
"""

IMMUTABLE = ("header", "features", "blobs", "blob_chunks", "blocks", "block_refs",
             "blob_refs", "nodes", "state_views", "commits", "manual_marks", "object_ids", "extensions")


def script():
    parts = [TABLES]
    keys = {"header": "singleton", "features": "name,version", "blobs": "sha",
            "blob_chunks": "sha,part", "blocks": "sha", "block_refs": "owner,target",
            "blob_refs": "owner,target", "nodes": "id", "commits": "id",
            "manual_marks": "commit_id", "extensions": "name", "state_views": "origin,target_schema,converter"}
    keys["object_ids"] = "kind,id"
    for table in IMMUTABLE:
        for operation in ("UPDATE", "DELETE"):
            parts.append("CREATE TRIGGER guard_%s_%s BEFORE %s ON %s BEGIN SELECT RAISE(ABORT,'immutable record'); END;"
                         % (table, operation.lower(), operation, table))
        predicate = " AND ".join("%s=NEW.%s" % (key, key) for key in keys[table].split(","))
        parts.append("CREATE TRIGGER guard_%s_replace BEFORE INSERT ON %s WHEN EXISTS(SELECT 1 FROM %s WHERE %s) BEGIN SELECT RAISE(ABORT,'duplicate immutable record'); END;"
                     % (table, table, table, predicate))
    parts.append("CREATE TRIGGER guard_control_delete BEFORE DELETE ON control BEGIN SELECT RAISE(ABORT,'control required'); END;")
    return "\n".join(parts)
