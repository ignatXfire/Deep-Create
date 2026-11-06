-- Executable C-03 contract fragment, NOT an installed application database.
-- One global head per installation. Full u64 is an 8-byte big-endian BLOB:
-- SQLite signed INTEGER cannot represent every domain RowVersion.
CREATE TABLE graph_topology_head (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    installation_id BLOB NOT NULL CHECK (length(installation_id) = 16),
    revision BLOB NOT NULL CHECK (
        length(revision) = 8 AND revision != x'0000000000000000'
    )
) STRICT;
