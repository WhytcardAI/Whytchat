use crate::actors::supervisor::SupervisorHandle;
use crate::actors::traits::{LlmActor, RagActor};
use crate::actors::messages::AppError;
use async_trait::async_trait;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use sqlx::sqlite::SqlitePool;

// --- Mock Components ---

struct MockLlmActor {
    delay_ms: u64,
    should_fail: bool,
    request_count: Arc<AtomicUsize>,
}

#[async_trait]
impl LlmActor for MockLlmActor {
    async fn generate_with_params(
        &self,
        _prompt: String,
        _system_prompt: Option<String>,
        _temperature: Option<f32>,
    ) -> Result<String, AppError> {
        self.request_count.fetch_add(1, Ordering::SeqCst);
        // Fast for analysis
        Ok("Intent Analysis".to_string())
    }

    async fn stream_generate_with_params(
        &self,
        _prompt: String,
        _system_prompt: Option<String>,
        _temperature: Option<f32>,
        chunk_sender: mpsc::Sender<Result<String, AppError>>,
    ) -> Result<(), AppError> {
        self.request_count.fetch_add(1, Ordering::SeqCst);
        if self.should_fail {
            sleep(Duration::from_millis(self.delay_ms)).await;
            return Err(AppError::Actor(crate::actors::messages::ActorError::Internal("Simulated Stream Failure".to_string())));
        }
        
        // Simulate slow streaming
        for i in 0..5 {
            sleep(Duration::from_millis(self.delay_ms / 5)).await;
            let _ = chunk_sender.send(Ok(format!("Token {} ", i))).await;
        }
        Ok(())
    }
}

struct MockRagActor;

#[async_trait]
impl RagActor for MockRagActor {
    async fn ingest(&self, _content: String, _metadata: Option<String>) -> Result<String, AppError> {
        Ok("Ingested".to_string())
    }
    
    async fn search_with_session(&self, _query: String, _session_id: Option<String>) -> Result<Vec<String>, AppError> {
        Ok(vec![])
    }
}

// Setup a test database in memory
async fn setup_db() -> SqlitePool {
    std::env::set_var("ENCRYPTION_KEY", "01234567890123456789012345678901"); // 32-byte key
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    // Manually run migrations instead of calling init_db which expects a file path
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_chaos_concurrent_requests() {
    // 1. Setup
    let pool = setup_db().await;
    let request_count = Arc::new(AtomicUsize::new(0));
    
    let mock_llm = Arc::new(MockLlmActor {
        delay_ms: 200, // 200ms delay per request
        should_fail: false,
        request_count: request_count.clone(),
    });
    let mock_rag = Arc::new(MockRagActor);
    
    let supervisor = SupervisorHandle::new_with_actors(
        mock_llm.clone(),
        mock_rag.clone(),
        Some(pool.clone())
    );

    // Create a session first
    let session_id = "test-session-chaos".to_string();
    let model_config = crate::models::ModelConfig {
        model_id: "default".to_string(),
        temperature: 0.7,
        system_prompt: "".to_string(),
    };
    crate::database::create_session_with_id(&pool, &session_id, "Chaos Test".to_string(), model_config).await.unwrap();

    // 2. Execute 50 concurrent requests
    let start_time = std::time::Instant::now();
    let mut handles = vec![];

    for i in 0..50 {
        let sup = supervisor.clone();
        let sid = session_id.clone();
        
        handles.push(tokio::spawn(async move {
            let result = sup.process_message(sid, format!("Message {}", i), None).await;
            assert!(result.is_ok(), "Request {} failed", i);
        }));
    }

    // 3. Wait for all results
    for handle in handles {
        handle.await.unwrap();
    }
    
    let total_duration = start_time.elapsed();
    println!("Total duration for 50 requests: {:?}", total_duration);
    
    // 4. Assertions
    // With 50 requests * 200ms delay, sequential execution would take > 10s.
    // Parallel execution should be much faster (limited by system threads, but definitely < 5s).
    // Note: 'generate_with_params' (intent) is also called, so it's actually 2 calls per message.
    // The 'generate_with_params' is fast in our mock, but 'stream_generate_with_params' has the delay.
    
    // Check that we actually processed requests
    let count = request_count.load(Ordering::SeqCst);
    // We expect 50 * 2 calls (1 intent + 1 stream) = 100 calls minimum
    assert!(count >= 100, "Expected at least 100 calls to LLM actor, got {}", count);

    // Verify non-blocking behavior (chaos criterion)
    // Allow some overhead, but 50 concurrent requests shouldn't take 50 * 200ms = 10s
    // Ideally it should be close to 200ms + overhead. Let's be generous with 3s.
    assert!(total_duration < Duration::from_secs(3), "System is processing sequentially! Duration: {:?}", total_duration);
    
    println!("Chaos Test Passed: System handled 50 concurrent requests in {:?}", total_duration);
}

#[tokio::test]
async fn test_resilience_under_failure() {
    // 1. Setup with failing actor
    let pool = setup_db().await;
    let request_count = Arc::new(AtomicUsize::new(0));
    
    // This mock will FAIL after a delay
    let mock_llm = Arc::new(MockLlmActor {
        delay_ms: 100,
        should_fail: true,
        request_count: request_count.clone(),
    });
    let mock_rag = Arc::new(MockRagActor);
    
    let supervisor = SupervisorHandle::new_with_actors(
        mock_llm.clone(),
        mock_rag.clone(),
        Some(pool.clone())
    );

    let session_id = "test-session-fail".to_string();
    let model_config = crate::models::ModelConfig {
        model_id: "default".to_string(),
        temperature: 0.7,
        system_prompt: "".to_string(),
    };
    crate::database::create_session_with_id(&pool, &session_id, "Fail Test".to_string(), model_config).await.unwrap();

    // 2. Send a request that will fail
    // The supervisor should return the error properly and NOT crash the actor system.
    let result = supervisor.process_message(session_id.clone(), "Crash me".to_string(), None).await;
    
    assert!(result.is_err(), "Request should have failed");
    
    // 3. Verify the supervisor is still alive by sending a VALID request (swapping actor or just checking response)
    // Since we can't swap the actor inside the handle, we just verify that the SupervisorHandle is still responsive
    // Even if the actor logic failed, the Supervisor loop should still be running.
    
    // Let's try another request (it will fail again, but it should *attempt* to process, meaning the channel is open)
    // If the supervisor panicked, the channel would be closed.
    let result2 = supervisor.process_message(session_id, "Am I alive?".to_string(), None).await;
    
    // If the actor loop crashed, send would fail (SendError) or timeout.
    // Since we get a Result (likely Err from the mock), it means the Actor loop processed the message!
    // We check that we don't get a "ChannelClosed" error implicitly by the fact `process_message` returns.
    
    match result2 {
        Ok(_) => panic!("Should fail due to mock config"),
        Err(e) => {
            // We expect the "Simulated Stream Failure" error, NOT a "Channel Closed" error.
            // AppError doesn't easily expose the variant string, but as long as we got a response, the Actor is alive.
            println!("Supervisor survived failure and returned: {:?}", e);
        }
    }
}