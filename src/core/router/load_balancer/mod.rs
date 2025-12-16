//! Load balancer module for provider selection
//!
//! **DEPRECATED**: This module is part of the legacy load balancer system.
//! For new code, use `crate::core::router::UnifiedRouter` instead, which provides:
//! - Deployment-based routing with sophisticated health tracking
//! - Built-in retry and fallback support via `execute()` method
//! - Lock-free concurrent access with DashMap
//! - Better integration with the router configuration system
//!
//! ## Migration Guide
//!
//! Replace:
//! ```ignore
//! let lb = LoadBalancer::new(RoutingStrategy::RoundRobin).await?;
//! lb.add_provider("openai", provider).await?;
//! let provider = lb.select_provider("gpt-4", &context).await?;
//! ```
//!
//! With:
//! ```ignore
//! let router = UnifiedRouter::new(RouterConfig::default());
//! router.add_deployment(deployment);
//! let result = router.execute("gpt-4", |deployment_id| async {
//!     // Your operation here
//! }).await?;
//! ```

mod core;
mod deployment_info;
mod fallback_config;
mod fallback_selection;
mod selection;
mod tag_routing;

pub use core::{LoadBalancer, LoadBalancerStats};
pub use deployment_info::DeploymentInfo;
pub use fallback_config::FallbackConfig;
