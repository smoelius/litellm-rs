//! Database storage implementation using SeaORM
//!
//! This module provides database connectivity and operations using SeaORM ORM.

// SeaORM implementation
/// Database entities module
pub mod entities;
/// Database migration module
pub mod migration;
/// SeaORM database implementation module
pub mod seaorm_db;

// Re-export the main database interface
pub use seaorm_db::SeaOrmDatabase as Database;
pub use seaorm_db::{DatabaseBackendType, DatabaseStats};
