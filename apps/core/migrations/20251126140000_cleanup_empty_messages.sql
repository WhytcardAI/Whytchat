-- Clean up empty messages that may have been created by previous bugs
DELETE FROM messages 
WHERE trim(content) = '' 
   OR content IS NULL 
   OR length(content) = 0;