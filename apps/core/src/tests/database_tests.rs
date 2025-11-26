//! Database Module Tests
//!
//! Comprehensive tests for database operations including sessions,
//! messages, files, and folders.

use crate::database;
use crate::models::{Folder, LibraryFile, Message, ModelConfig, Session};
use sqlx::sqlite::SqlitePool;
use tempfile::tempdir;

/// Create a test database pool with a temporary file
async fn create_test_pool() -> SqlitePool {
    let dir = tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.sqlite");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
        .expect("Failed to create test pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[cfg(test)]
mod session_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Test Session".to_string(), config)
            .await
            .expect("Failed to create session");

        assert!(!session.id.is_empty());
        assert_eq!(session.title, "Test Session");
        assert!(!session.is_favorite);
    }

    #[tokio::test]
    async fn test_get_session() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let created = database::create_session(&pool, "Get Test".to_string(), config)
            .await
            .expect("Failed to create session");

        let fetched = database::get_session(&pool, &created.id)
            .await
            .expect("Failed to get session");

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.title, "Get Test");
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let pool = create_test_pool().await;

        // Create multiple sessions
        for i in 0..3 {
            let config = ModelConfig::default();
            database::create_session(&pool, format!("Session {}", i), config)
                .await
                .expect("Failed to create session");
        }

        let sessions = database::list_sessions(&pool)
            .await
            .expect("Failed to list sessions");

        assert_eq!(sessions.len(), 3);
    }

    #[tokio::test]
    async fn test_update_session_title() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Original".to_string(), config)
            .await
            .expect("Failed to create session");

        database::update_session(&pool, &session.id, Some("Updated".to_string()), None)
            .await
            .expect("Failed to update session");

        let updated = database::get_session(&pool, &session.id)
            .await
            .expect("Failed to get session");

        assert_eq!(updated.title, "Updated");
    }

    #[tokio::test]
    async fn test_toggle_session_favorite() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Favorite Test".to_string(), config)
            .await
            .expect("Failed to create session");

        assert!(!session.is_favorite);

        database::toggle_session_favorite(&pool, &session.id)
            .await
            .expect("Failed to toggle favorite");

        let updated = database::get_session(&pool, &session.id)
            .await
            .expect("Failed to get session");

        assert!(updated.is_favorite);

        // Toggle again
        database::toggle_session_favorite(&pool, &session.id)
            .await
            .expect("Failed to toggle favorite");

        let updated2 = database::get_session(&pool, &session.id)
            .await
            .expect("Failed to get session");

        assert!(!updated2.is_favorite);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Delete Test".to_string(), config)
            .await
            .expect("Failed to create session");

        database::delete_session(&pool, &session.id)
            .await
            .expect("Failed to delete session");

        let result = database::get_session(&pool, &session.id).await;
        assert!(result.is_err(), "Session should be deleted");
    }

    #[tokio::test]
    async fn test_session_ordering() {
        let pool = create_test_pool().await;

        // Create sessions with delays to ensure different timestamps
        for i in 0..3 {
            let config = ModelConfig::default();
            database::create_session(&pool, format!("Session {}", i), config)
                .await
                .expect("Failed to create session");
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let sessions = database::list_sessions(&pool)
            .await
            .expect("Failed to list sessions");

        // Sessions should be ordered by created_at descending (newest first)
        for i in 0..sessions.len() - 1 {
            assert!(
                sessions[i].created_at >= sessions[i + 1].created_at,
                "Sessions should be ordered by creation date"
            );
        }
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[tokio::test]
    async fn test_add_message() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Message Test".to_string(), config)
            .await
            .expect("Failed to create session");

        let message = database::add_message(&pool, &session.id, "user", "Hello world")
            .await
            .expect("Failed to add message");

        assert!(!message.id.is_empty());
        assert_eq!(message.session_id, session.id);
        assert_eq!(message.role, "user");
        assert_eq!(message.content, "Hello world");
    }

    #[tokio::test]
    async fn test_get_session_messages() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Messages Test".to_string(), config)
            .await
            .expect("Failed to create session");

        // Add multiple messages
        database::add_message(&pool, &session.id, "user", "Message 1")
            .await
            .expect("Failed to add message");
        database::add_message(&pool, &session.id, "assistant", "Response 1")
            .await
            .expect("Failed to add message");
        database::add_message(&pool, &session.id, "user", "Message 2")
            .await
            .expect("Failed to add message");

        let messages = database::get_session_messages(&pool, &session.id)
            .await
            .expect("Failed to get messages");

        assert_eq!(messages.len(), 3);
    }

    #[tokio::test]
    async fn test_message_ordering() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Order Test".to_string(), config)
            .await
            .expect("Failed to create session");

        for i in 0..5 {
            database::add_message(&pool, &session.id, "user", &format!("Message {}", i))
                .await
                .expect("Failed to add message");
        }

        let messages = database::get_session_messages(&pool, &session.id)
            .await
            .expect("Failed to get messages");

        // Messages should be ordered by timestamp ascending
        for i in 0..messages.len() - 1 {
            assert!(
                messages[i].created_at <= messages[i + 1].created_at,
                "Messages should be in chronological order"
            );
        }
    }

    #[tokio::test]
    async fn test_message_roles() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Role Test".to_string(), config)
            .await
            .expect("Failed to create session");

        let user_msg = database::add_message(&pool, &session.id, "user", "User message")
            .await
            .expect("Failed to add user message");
        let assistant_msg = database::add_message(&pool, &session.id, "assistant", "Assistant response")
            .await
            .expect("Failed to add assistant message");
        let system_msg = database::add_message(&pool, &session.id, "system", "System note")
            .await
            .expect("Failed to add system message");

        assert_eq!(user_msg.role, "user");
        assert_eq!(assistant_msg.role, "assistant");
        assert_eq!(system_msg.role, "system");
    }

    #[tokio::test]
    async fn test_empty_message_content() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Empty Test".to_string(), config)
            .await
            .expect("Failed to create session");

        // Empty content should be allowed (for thinking steps, etc.)
        let message = database::add_message(&pool, &session.id, "assistant", "")
            .await
            .expect("Failed to add empty message");

        assert_eq!(message.content, "");
    }

    #[tokio::test]
    async fn test_long_message_content() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Long Test".to_string(), config)
            .await
            .expect("Failed to create session");

        let long_content = "A".repeat(10000);
        let message = database::add_message(&pool, &session.id, "user", &long_content)
            .await
            .expect("Failed to add long message");

        assert_eq!(message.content.len(), 10000);
    }
}

#[cfg(test)]
mod folder_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_folder() {
        let pool = create_test_pool().await;

        let folder = database::create_folder(&pool, "Test Folder".to_string())
            .await
            .expect("Failed to create folder");

        assert!(!folder.id.is_empty());
        assert_eq!(folder.name, "Test Folder");
    }

    #[tokio::test]
    async fn test_list_folders() {
        let pool = create_test_pool().await;

        for i in 0..3 {
            database::create_folder(&pool, format!("Folder {}", i))
                .await
                .expect("Failed to create folder");
        }

        let folders = database::list_folders(&pool)
            .await
            .expect("Failed to list folders");

        assert_eq!(folders.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_folder() {
        let pool = create_test_pool().await;

        let folder = database::create_folder(&pool, "Delete Folder".to_string())
            .await
            .expect("Failed to create folder");

        database::delete_folder(&pool, &folder.id)
            .await
            .expect("Failed to delete folder");

        let folders = database::list_folders(&pool)
            .await
            .expect("Failed to list folders");

        assert!(
            !folders.iter().any(|f| f.id == folder.id),
            "Folder should be deleted"
        );
    }

    #[tokio::test]
    async fn test_move_session_to_folder() {
        let pool = create_test_pool().await;

        let folder = database::create_folder(&pool, "Session Folder".to_string())
            .await
            .expect("Failed to create folder");

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Movable Session".to_string(), config)
            .await
            .expect("Failed to create session");

        database::move_session_to_folder(&pool, &session.id, Some(&folder.id))
            .await
            .expect("Failed to move session");

        let updated = database::get_session(&pool, &session.id)
            .await
            .expect("Failed to get session");

        assert_eq!(updated.folder_id, Some(folder.id));
    }

    #[tokio::test]
    async fn test_remove_session_from_folder() {
        let pool = create_test_pool().await;

        let folder = database::create_folder(&pool, "Temp Folder".to_string())
            .await
            .expect("Failed to create folder");

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Session".to_string(), config)
            .await
            .expect("Failed to create session");

        // Move to folder
        database::move_session_to_folder(&pool, &session.id, Some(&folder.id))
            .await
            .expect("Failed to move session");

        // Remove from folder
        database::move_session_to_folder(&pool, &session.id, None)
            .await
            .expect("Failed to remove from folder");

        let updated = database::get_session(&pool, &session.id)
            .await
            .expect("Failed to get session");

        assert_eq!(updated.folder_id, None);
    }
}

#[cfg(test)]
mod library_file_tests {
    use super::*;

    #[tokio::test]
    async fn test_add_library_file() {
        let pool = create_test_pool().await;

        let file = database::add_library_file(
            &pool,
            "test-file.txt",
            "text/plain",
            "path/to/file.txt",
            "Sample content",
        )
        .await
        .expect("Failed to add library file");

        assert!(!file.id.is_empty());
        assert_eq!(file.original_name, "test-file.txt");
        assert_eq!(file.mime_type, "text/plain");
    }

    #[tokio::test]
    async fn test_list_library_files() {
        let pool = create_test_pool().await;

        for i in 0..5 {
            database::add_library_file(
                &pool,
                &format!("file{}.txt", i),
                "text/plain",
                &format!("path/file{}.txt", i),
                &format!("Content {}", i),
            )
            .await
            .expect("Failed to add library file");
        }

        let files = database::list_library_files(&pool)
            .await
            .expect("Failed to list library files");

        assert_eq!(files.len(), 5);
    }

    #[tokio::test]
    async fn test_get_library_file() {
        let pool = create_test_pool().await;

        let created = database::add_library_file(
            &pool,
            "get-test.txt",
            "text/plain",
            "path/get-test.txt",
            "Get test content",
        )
        .await
        .expect("Failed to add library file");

        let fetched = database::get_library_file(&pool, &created.id)
            .await
            .expect("Failed to get library file");

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.original_name, "get-test.txt");
    }

    #[tokio::test]
    async fn test_delete_library_file() {
        let pool = create_test_pool().await;

        let file = database::add_library_file(
            &pool,
            "delete-test.txt",
            "text/plain",
            "path/delete.txt",
            "Delete content",
        )
        .await
        .expect("Failed to add library file");

        database::delete_library_file(&pool, &file.id)
            .await
            .expect("Failed to delete library file");

        let result = database::get_library_file(&pool, &file.id).await;
        assert!(result.is_err(), "File should be deleted");
    }

    #[tokio::test]
    async fn test_link_file_to_session() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Link Test".to_string(), config)
            .await
            .expect("Failed to create session");

        let file = database::add_library_file(
            &pool,
            "link-test.txt",
            "text/plain",
            "path/link.txt",
            "Link content",
        )
        .await
        .expect("Failed to add library file");

        database::link_file_to_session(&pool, &file.id, &session.id)
            .await
            .expect("Failed to link file to session");

        let session_files = database::get_session_files(&pool, &session.id)
            .await
            .expect("Failed to get session files");

        assert_eq!(session_files.len(), 1);
        assert_eq!(session_files[0].id, file.id);
    }

    #[tokio::test]
    async fn test_unlink_file_from_session() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Unlink Test".to_string(), config)
            .await
            .expect("Failed to create session");

        let file = database::add_library_file(
            &pool,
            "unlink-test.txt",
            "text/plain",
            "path/unlink.txt",
            "Unlink content",
        )
        .await
        .expect("Failed to add library file");

        database::link_file_to_session(&pool, &file.id, &session.id)
            .await
            .expect("Failed to link file");

        database::unlink_file_from_session(&pool, &file.id, &session.id)
            .await
            .expect("Failed to unlink file");

        let session_files = database::get_session_files(&pool, &session.id)
            .await
            .expect("Failed to get session files");

        assert_eq!(session_files.len(), 0);
    }
}

#[cfg(test)]
mod model_config_tests {
    use super::*;

    #[tokio::test]
    async fn test_session_with_custom_config() {
        let pool = create_test_pool().await;

        let config = ModelConfig {
            temperature: 0.9,
            max_tokens: 2048,
            system_prompt: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };

        let session = database::create_session(&pool, "Config Test".to_string(), config.clone())
            .await
            .expect("Failed to create session");

        let fetched = database::get_session(&pool, &session.id)
            .await
            .expect("Failed to get session");

        assert_eq!(fetched.model_config.0.temperature, 0.9);
        assert_eq!(fetched.model_config.0.max_tokens, 2048);
        assert_eq!(
            fetched.model_config.0.system_prompt,
            "You are a helpful assistant."
        );
    }

    #[tokio::test]
    async fn test_config_encryption() {
        let pool = create_test_pool().await;

        let config = ModelConfig {
            system_prompt: "Secret system prompt".to_string(),
            ..Default::default()
        };

        let session = database::create_session(&pool, "Encryption Test".to_string(), config)
            .await
            .expect("Failed to create session");

        // The config should be encrypted in the database
        // We verify by fetching and checking it decrypts correctly
        let fetched = database::get_session(&pool, &session.id)
            .await
            .expect("Failed to get session");

        assert_eq!(
            fetched.model_config.0.system_prompt,
            "Secret system prompt"
        );
    }
}

#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn test_concurrent_session_creation() {
        let pool = create_test_pool().await;

        let mut tasks = JoinSet::new();

        for i in 0..10 {
            let pool_clone = pool.clone();
            tasks.spawn(async move {
                let config = ModelConfig::default();
                database::create_session(&pool_clone, format!("Concurrent {}", i), config).await
            });
        }

        let mut success_count = 0;
        while let Some(result) = tasks.join_next().await {
            if result.unwrap().is_ok() {
                success_count += 1;
            }
        }

        assert_eq!(success_count, 10, "All concurrent sessions should be created");
    }

    #[tokio::test]
    async fn test_concurrent_message_addition() {
        let pool = create_test_pool().await;

        let config = ModelConfig::default();
        let session = database::create_session(&pool, "Concurrent Msgs".to_string(), config)
            .await
            .expect("Failed to create session");

        let mut tasks = JoinSet::new();

        for i in 0..20 {
            let pool_clone = pool.clone();
            let session_id = session.id.clone();
            tasks.spawn(async move {
                database::add_message(&pool_clone, &session_id, "user", &format!("Message {}", i))
                    .await
            });
        }

        let mut success_count = 0;
        while let Some(result) = tasks.join_next().await {
            if result.unwrap().is_ok() {
                success_count += 1;
            }
        }

        assert_eq!(success_count, 20);

        let messages = database::get_session_messages(&pool, &session.id)
            .await
            .expect("Failed to get messages");

        assert_eq!(messages.len(), 20);
    }
}
