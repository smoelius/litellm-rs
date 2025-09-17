//! Provider Registry
//!
//! Centralized registry for managing Provider enum instances

use super::{Provider, ProviderType};
use std::collections::HashMap;

/// Provider Registry using enum-based providers
pub struct ProviderRegistry {
    providers: HashMap<String, Provider>,
}

impl ProviderRegistry {
    /// Create new provider registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a provider
    pub fn register(&mut self, provider: Provider) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }

    /// Get provider by name
    pub fn get(&self, name: &str) -> Option<&Provider> {
        self.providers.get(name)
    }

    /// Get mutable provider by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Provider> {
        self.providers.get_mut(name)
    }

    /// List all registered providers
    pub fn list(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Remove provider
    pub fn remove(&mut self, name: &str) -> Option<Provider> {
        self.providers.remove(name)
    }

    /// Check if provider is registered
    pub fn contains(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Get provider count
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// Clear all providers
    pub fn clear(&mut self) {
        self.providers.clear();
    }

    /// Get providers by type
    pub fn get_by_type(&self, provider_type: ProviderType) -> Vec<&Provider> {
        self.providers
            .values()
            .filter(|p| p.provider_type() == provider_type)
            .collect()
    }

    /// Find providers supporting a specific model
    pub fn find_supporting_model(&self, model: &str) -> Vec<&Provider> {
        self.providers
            .values()
            .filter(|p| p.supports_model(model))
            .collect()
    }

    /// Get all providers as a vector
    pub fn all(&self) -> Vec<&Provider> {
        self.providers.values().collect()
    }

    /// Compatibility method for get_provider (alias for get)
    pub fn get_provider(&self, name: &str) -> Option<&Provider> {
        self.get(name)
    }

    /// Compatibility method for get_all_providers (alias for all)
    pub fn get_all_providers(&self) -> Vec<&Provider> {
        self.all()
    }

    /// Get provider values iterator (for compatibility with HashMap iteration)
    pub fn values(&self) -> impl Iterator<Item = &Provider> {
        self.providers.values()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ProviderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderRegistry")
            .field("provider_count", &self.providers.len())
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .finish()
    }
}
