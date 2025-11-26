-- Add type to folders (default 'session')
ALTER TABLE folders ADD COLUMN type TEXT NOT NULL DEFAULT 'session';

-- Add folder_id to library_files
ALTER TABLE library_files ADD COLUMN folder_id TEXT REFERENCES folders(id);
