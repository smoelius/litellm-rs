# Unified Router Implementation Plan

> Date: 2025-12-11
> Goal: 统一现有 3 个 Router 版本为一个高性能 Router

## 1. Current State Analysis

### Components (All exist but NOT integrated)

| Component | File | Status |
|-----------|------|--------|
| `Router` | `router/mod.rs` | ❌ Not used by server |
| `RouterV2` | `router/router_v2.rs` | ❌ Not used, TODO in select |
| `DefaultRouter` | `completion.rs` | ✅ Used by completion() API |
| `LoadBalancer` | `router/load_balancer.rs` | ❌ Not used |
| `HealthChecker` | `router/health.rs` | ❌ Not used |
| `StrategyExecutor` | `router/strategy.rs` | ❌ Not used |
| `CircuitBreaker` | `utils/error/recovery.rs` | ❌ Not used |
| `ProviderRegistry` | `providers/provider_registry.rs` | ✅ Used by server |

### Server Current Flow
```
AppState.router = Arc<ProviderRegistry>  // Simple HashMap<String, Provider>
                    ↓
            No load balancing
            No health checking
            No circuit breaker
            No strategy selection
```

## 2. Target Architecture

### Single Unified Router
```rust
pub struct Router {
    // Core: Deployment management (DashMap for lock-free concurrent access)
    deployments: DashMap<String, Deployment>,

    // Model routing: model -> deployment_ids
    model_index: DashMap<String, Vec<String>>,

    // Model group aliases: "gpt4" -> "gpt-4"
    model_aliases: DashMap<String, String>,

    // Strategy executor (already implemented in strategy.rs)
    strategy: Arc<StrategyExecutor>,

    // Fallback config (already implemented in load_balancer.rs)
    fallback_config: FallbackConfig,

    // Metrics (already implemented in metrics.rs)
    metrics: Arc<RouterMetrics>,
}

pub struct Deployment {
    pub id: String,
    pub provider: Provider,           // enum Provider - static dispatch
    pub model_name: String,           // actual model name
    pub model_group: String,          // logical group (e.g., "gpt-4")
    pub info: DeploymentInfo,         // tags, priority, metadata
    pub circuit_breaker: CircuitBreaker,  // per-deployment breaker
    pub health: AtomicU8,             // 0=unknown, 1=healthy, 2=degraded, 3=unhealthy
    pub usage: DeploymentUsage,       // TPM/RPM/active tracking
}

pub struct DeploymentUsage {
    pub tpm: AtomicU64,               // tokens this minute
    pub rpm: AtomicU64,               // requests this minute
    pub active: AtomicUsize,          // active concurrent requests
    pub tpm_limit: Option<u64>,
    pub rpm_limit: Option<u64>,
    pub last_reset: AtomicU64,        // timestamp for minute window
}
```

### Performance Characteristics
- **Zero-copy model lookup**: DashMap O(1)
- **Lock-free counters**: AtomicU64/AtomicUsize
- **Static dispatch**: Provider enum (no vtable)
- **Minimal allocations**: Arc sharing, no cloning providers

## 3. Implementation Plan

### Phase 1: Core Router Structure (P0)

**File: `src/core/router/unified.rs`** (NEW)

```rust
// 1. Deployment struct
pub struct Deployment { ... }

// 2. DeploymentUsage with atomic counters
pub struct DeploymentUsage { ... }

// 3. Router struct with DashMap
pub struct Router { ... }

// 4. Core methods:
impl Router {
    pub fn new(config: RouterConfig) -> Self;
    pub fn register_deployment(&self, deployment: Deployment);
    pub fn remove_deployment(&self, id: &str);

    // Main routing method
    pub async fn route<R, F>(&self, model: &str, context: &RequestContext, f: F) -> Result<R>
    where F: FnOnce(&Provider) -> Future<Output = Result<R>>;
}
```

### Phase 2: Integrate Existing Components (P1)

**Reuse existing code (no rewrite):**

1. **StrategyExecutor** from `strategy.rs` - use as-is
2. **CircuitBreaker** from `recovery.rs` - embed in Deployment
3. **FallbackConfig** from `load_balancer.rs` - use as-is
4. **RouterMetrics** from `metrics.rs` - use as-is
5. **DeploymentInfo** from `load_balancer.rs` - use as-is

### Phase 3: Replace Usages (P2)

1. **server/mod.rs**:
   ```rust
   // Before:
   pub router: Arc<ProviderRegistry>,

   // After:
   pub router: Arc<Router>,
   ```

2. **completion.rs DefaultRouter**:
   - Keep for Python LiteLLM API compatibility
   - Delegate to unified Router internally

3. **Delete deprecated files**:
   - `router/router_v2.rs` - merge into unified
   - Parts of `router/mod.rs` - keep only unified Router

### Phase 4: Add Missing Features (P3)

1. **Background health checker** (from health.rs)
2. **Minute-based usage reset** task
3. **Warmup/preload** cache for model->deployment mapping

## 4. API Design

### Registration API
```rust
// Simple registration
router.register_provider("openai-1", Provider::OpenAI(openai_provider));

// With full deployment config
router.register_deployment(Deployment {
    id: "openai-gpt4-primary".into(),
    provider: Provider::OpenAI(openai_provider),
    model_name: "gpt-4-turbo".into(),
    model_group: "gpt-4".into(),
    info: DeploymentInfo::new()
        .with_tags(["production", "fast"])
        .with_priority(0),
    circuit_breaker: CircuitBreaker::new(CircuitBreakerConfig::default()),
    health: AtomicU8::new(1), // healthy
    usage: DeploymentUsage::default(),
});
```

### Routing API
```rust
// Basic routing
let response = router.chat_completion(model, request, context).await?;

// With options
let response = router.chat_completion_with_options(
    model,
    request,
    context,
    RoutingOptions {
        tags: Some(vec!["fast"]),
        fallback_models: Some(vec!["gpt-3.5-turbo"]),
        timeout: Some(Duration::from_secs(30)),
    }
).await?;

// Low-level routing
let deployment = router.select_deployment(model, context).await?;
let response = deployment.provider.chat_completion(request, context).await?;
router.record_success(&deployment.id, response.usage.total_tokens).await;
```

## 5. Configuration

### YAML Config (Optional)
```yaml
router:
  strategy: lowest_latency
  fallback:
    general:
      "gpt-4": ["gpt-4-turbo", "gpt-3.5-turbo"]
    context_window:
      "gpt-4": ["gpt-4-32k", "claude-3-opus"]

  model_groups:
    - name: gpt-4
      aliases: ["gpt4", "openai/gpt-4"]
      deployments:
        - id: openai-primary
          provider: openai
          model: gpt-4-turbo
          priority: 0
          tags: ["production"]
          rate_limits:
            tpm: 100000
            rpm: 500
        - id: azure-backup
          provider: azure
          model: gpt-4
          priority: 1
          tags: ["backup"]
```

## 6. Migration Path

### Step 1: Create unified.rs (non-breaking)
- New file, doesn't touch existing code
- Full test coverage

### Step 2: Update server/mod.rs
- Switch AppState.router to new Router
- Keep ProviderRegistry as fallback

### Step 3: Update completion.rs
- DefaultRouter delegates to Router
- Maintains Python LiteLLM API

### Step 4: Cleanup
- Remove router_v2.rs
- Simplify router/mod.rs
- Update documentation

## 7. Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| Route selection | < 1μs | DashMap + atomic ops |
| Memory per deployment | < 1KB | No heap allocation in hot path |
| Concurrent requests | 100K+ | Lock-free design |
| Strategy selection | < 10μs | Pre-computed metrics |

## 8. Testing Strategy

1. **Unit tests**: Each component in isolation
2. **Integration tests**: Full routing flow
3. **Benchmark tests**: Performance regression
4. **Load tests**: Concurrent access patterns

## 9. Files to Modify

| File | Action |
|------|--------|
| `src/core/router/unified.rs` | NEW - main Router |
| `src/core/router/mod.rs` | UPDATE - re-export unified |
| `src/core/router/router_v2.rs` | DELETE |
| `src/server/mod.rs` | UPDATE - use new Router |
| `src/core/completion.rs` | UPDATE - delegate to Router |
| `src/lib.rs` | UPDATE - export new Router |

## 10. Estimated Effort

| Phase | Effort | Notes |
|-------|--------|-------|
| Phase 1 | 2-3 hours | Core structure |
| Phase 2 | 1-2 hours | Integration (reuse code) |
| Phase 3 | 1 hour | Replace usages |
| Phase 4 | 2 hours | Background tasks |
| Testing | 2 hours | Comprehensive tests |
| **Total** | **8-10 hours** | |

---

## Next Steps

1. Review and approve this plan
2. Start with Phase 1: Core Router Structure
3. Incremental PRs for each phase
