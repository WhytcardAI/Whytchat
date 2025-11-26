-- Migration: Add sorting and organization fields to sessions
-- Add is_favorite for pinning sessions
ALTER TABLE sessions ADD COLUMN is_favorite BOOLEAN NOT NULL DEFAULT 0;

-- Add folder_id for organizing sessions into folders (nullable)
ALTER TABLE sessions ADD COLUMN folder_id TEXT DEFAULT NULL;

-- Add sort_order for manual ordering within folders
ALTER TABLE sessions ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;

-- Add updated_at for "last modified" sorting
ALTER TABLE sessions ADD COLUMN updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- Create folders table for session organization
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT DEFAULT '#6366f1', -- default violet
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for faster folder queries
CREATE INDEX IF NOT EXISTS idx_sessions_folder ON sessions(folder_id);
CREATE INDEX IF NOT EXISTS idx_sessions_favorite ON sessions(is_favorite);
CREATE INDEX IF NOT EXISTS idx_sessions_updated ON sessions(updated_at);
