-- Add an index to the session_id column on the messages table to speed up message retrieval for a given session.
CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages (session_id);
