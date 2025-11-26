//! Test Module
//!
//! Comprehensive test suite for WhytChat backend.
//!
//! ## Test Categories
//! - `brain_tests`: Intent classification, keyword extraction, complexity scoring
//! - `database_tests`: CRUD operations for sessions, messages, files, folders
//! - `actor_tests`: LLM, RAG, and Supervisor actor behavior
//! - `encryption_tests`: AES-256-GCM encryption/decryption
//! - `text_extract_tests`: PDF, DOCX, and text extraction
//! - `supervisor_tests`: Supervisor orchestration
//! - `chaos_test`: Chaos engineering and resilience tests
//! - `integration_tests`: Full workflow integration tests

pub mod supervisor_tests;
pub mod chaos_test;
mod download_tests;
pub mod brain_tests;
pub mod database_tests;
pub mod actor_tests;
pub mod encryption_tests;
pub mod text_extract_tests;
pub mod integration_tests;
