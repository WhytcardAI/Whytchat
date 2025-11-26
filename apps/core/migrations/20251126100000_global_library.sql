-- Migration to Global Library Architecture

-- 1. Create the global library table
CREATE TABLE library_files (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    file_type TEXT NOT NULL,
    size INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 2. Create the many-to-many link table
CREATE TABLE session_files_link (
    session_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    attached_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (session_id, file_id),
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY(file_id) REFERENCES library_files(id) ON DELETE CASCADE
);

-- 3. Migrate existing data
-- We assume existing file_path contains the name at the end.
-- We insert into library_files using the existing ID.
INSERT INTO library_files (id, name, path, file_type, created_at)
SELECT
    id,
    replace(file_path, session_id || '/', ''), -- Attempt to extract name (naive but works for standard uploads)
    file_path,
    file_type,
    added_at
FROM session_files;

-- Link them to their original sessions
INSERT INTO session_files_link (session_id, file_id, attached_at)
SELECT session_id, id, added_at FROM session_files;

-- 4. Drop the old table
DROP TABLE session_files;
