# LiteLLM-RS Router 完整重构方案

> 目标：实现 LiteLLM Python Router 的全部功能，性能更强
>
> **状态：✅ 已完成** (2025-12-11)

## 实现总结

### 已完成功能

| 功能 | 状态 | 测试 |
|------|------|------|
| Deployment 核心结构 | ✅ | 10 tests |
| Router 基础结构 | ✅ | 16 tests |
| 7 种路由策略 | ✅ | 14 tests |
| Cooldown 机制 | ✅ | 21 tests |
| Fallback 机制 | ✅ | 19 tests |
| Execute 流程 (retry + fallback) | ✅ | 13 tests |
| Server 集成 | ✅ | - |
| **总计** | **84 unified tests** | **774+ total tests** |

### 新文件

- `src/core/router/deployment.rs` - Deployment 核心结构
- `src/core/router/unified.rs` - 统一 Router 实现

### 删除文件

- `src/core/router/router_v2.rs` - 废弃的实验性实现

---

## 1. LiteLLM Python Router 功能清单

### 核心功能 (必须实现)

| 功能 | Python 实现 | 说明 |
|------|-------------|------|
| **Model List** | `model_list` | 一个 model_name 对应多个 deployment |
| **Routing Strategies** | 6 种策略 | simple-shuffle, least-busy, usage-based, latency-based, cost-based, rate-limit-aware |
| **Fallbacks** | 4 种类型 | general, context_window, content_policy, rate_limit |
| **Cooldown** | `CooldownCache` | 失败后冷却，自动恢复 |
| **Rate Limiting** | TPM/RPM | per-deployment 限制 |
| **Retries** | `num_retries` | 重试 + 指数退避 |
| **Health Tracking** | 统计 | success/fail/total calls |
| **Pre-call Checks** | context window | 调用前验证 |
| **Concurrent Limit** | `max_parallel_requests` | per-deployment 并发限制 |
| **Weighted Selection** | `weight` | 加权随机 |
| **Tag Filtering** | `tags` | 按 tag 过滤 deployment |

### Python Router 配置参数

```python
Router(
    model_list=[...],                    # deployment 列表
    routing_strategy="simple-shuffle",   # 路由策略
    num_retries=3,                       # 重试次数
    retry_after=0,                       # 重试最小间隔(秒)
    allowed_fails=3,                     # 冷却前允许失败次数
    cooldown_time=5,                     # 冷却时间(秒)
    timeout=60,                          # 超时时间
    fallbacks=[...],                     # 通用 fallback
    context_window_fallbacks=[...],      # context 超限 fallback
    content_policy_fallbacks=[...],      # 内容策略 fallback
    enable_pre_call_checks=True,         # 启用预检查
    cache_responses=False,               # 响应缓存
)
```

### Python model_list 结构

```python
{
    "model_name": "gpt-4",              # 用户调用的名称
    "litellm_params": {
        "model": "azure/gpt-4-turbo",   # 实际模型
        "api_key": "...",
        "api_base": "...",
        "rpm": 500,                      # requests/minute 限制
        "tpm": 100000,                   # tokens/minute 限制
        "weight": 1,                     # 权重
        "max_parallel_requests": 10,     # 最大并发
    },
    "model_info": {
        "id": "deployment-1",
    },
    "tags": ["production", "fast"],
}
```

---

## 2. Rust Router 架构设计

### 2.1 核心数据结构

```rust
/// Router - 统一路由器
pub struct Router {
    /// 所有 deployments (lock-free)
    deployments: DashMap<DeploymentId, Deployment>,

    /// model_name -> deployment_ids 索引
    model_index: DashMap<ModelName, Vec<DeploymentId>>,

    /// model_name 别名: "gpt4" -> "gpt-4"
    model_aliases: DashMap<String, ModelName>,

    /// 路由策略
    strategy: RoutingStrategy,

    /// Fallback 配置
    fallbacks: FallbackConfig,

    /// 全局配置
    config: RouterConfig,

    /// 统计数据
    stats: RouterStats,
}

/// Deployment - 一个具体的 provider 部署
pub struct Deployment {
    /// 唯一 ID
    pub id: DeploymentId,

    /// Provider 实例 (enum, 静态派发)
    pub provider: Provider,

    /// 实际模型名 (e.g., "azure/gpt-4-turbo")
    pub model: String,

    /// 用户调用的名称 (e.g., "gpt-4")
    pub model_name: ModelName,

    /// 配置
    pub config: DeploymentConfig,

    /// 状态 (lock-free)
    pub state: DeploymentState,

    /// Tags
    pub tags: Vec<String>,
}

/// Deployment 配置
pub struct DeploymentConfig {
    /// Tokens per minute 限制
    pub tpm_limit: Option<u64>,

    /// Requests per minute 限制
    pub rpm_limit: Option<u64>,

    /// 最大并发请求
    pub max_parallel_requests: Option<u32>,

    /// 权重 (用于 weighted random)
    pub weight: u32,

    /// 超时 (秒)
    pub timeout: u64,

    /// 优先级 (lower = higher priority)
    pub priority: u32,
}

/// Deployment 运行时状态 (全部 atomic, lock-free)
pub struct DeploymentState {
    /// 健康状态: 0=unknown, 1=healthy, 2=degraded, 3=unhealthy, 4=cooldown
    pub health: AtomicU8,

    /// 当前分钟 TPM 使用量
    pub tpm_current: AtomicU64,

    /// 当前分钟 RPM 使用量
    pub rpm_current: AtomicU64,

    /// 当前活跃请求数
    pub active_requests: AtomicU32,

    /// 总请求数
    pub total_requests: AtomicU64,

    /// 成功请求数
    pub success_requests: AtomicU64,

    /// 失败请求数
    pub fail_requests: AtomicU64,

    /// 当前分钟失败数 (用于 cooldown 判断)
    pub fails_this_minute: AtomicU32,

    /// 冷却结束时间 (unix timestamp)
    pub cooldown_until: AtomicU64,

    /// 最后请求时间
    pub last_request_at: AtomicU64,

    /// 平均延迟 (微秒, 滑动窗口)
    pub avg_latency_us: AtomicU64,

    /// 上次分钟重置时间
    pub minute_reset_at: AtomicU64,
}

/// 路由策略
#[derive(Clone, Copy)]
pub enum RoutingStrategy {
    /// 简单随机 (考虑权重)
    SimpleShufle,

    /// 最少并发
    LeastBusy,

    /// 最低 TPM 使用率
    UsageBased,

    /// 最低延迟
    LatencyBased,

    /// 最低成本
    CostBased,

    /// 限流感知 (避免超限)
    RateLimitAware,

    /// Round Robin
    RoundRobin,
}

/// Fallback 配置
pub struct FallbackConfig {
    /// 通用 fallback: model_name -> fallback model_names
    pub general: HashMap<ModelName, Vec<ModelName>>,

    /// Context window 超限 fallback
    pub context_window: HashMap<ModelName, Vec<ModelName>>,

    /// 内容策略 fallback
    pub content_policy: HashMap<ModelName, Vec<ModelName>>,

    /// Rate limit fallback
    pub rate_limit: HashMap<ModelName, Vec<ModelName>>,
}

/// Router 配置
pub struct RouterConfig {
    /// 重试次数
    pub num_retries: u32,

    /// 重试最小间隔 (秒)
    pub retry_after: u64,

    /// 冷却前允许失败次数
    pub allowed_fails: u32,

    /// 冷却时间 (秒)
    pub cooldown_time: u64,

    /// 默认超时
    pub timeout: u64,

    /// 最大 fallback 次数
    pub max_fallbacks: u32,

    /// 启用预检查
    pub enable_pre_call_checks: bool,
}

/// 类型别名
pub type DeploymentId = String;
pub type ModelName = String;
```

### 2.2 Router 主要方法

```rust
impl Router {
    // ========== 构造 ==========

    /// 创建 Router
    pub fn new(config: RouterConfig) -> Self;

    /// 从 YAML 配置创建
    pub async fn from_config(path: &str) -> Result<Self>;

    // ========== Deployment 管理 ==========

    /// 添加 deployment
    pub fn add_deployment(&self, deployment: Deployment);

    /// 移除 deployment
    pub fn remove_deployment(&self, id: &DeploymentId) -> Option<Deployment>;

    /// 更新 deployment
    pub fn upsert_deployment(&self, deployment: Deployment);

    /// 设置 model_list (批量)
    pub fn set_model_list(&self, deployments: Vec<Deployment>);

    // ========== 路由核心 ==========

    /// 获取可用 deployment (核心路由方法)
    pub async fn get_available_deployment(
        &self,
        model_name: &str,
        context: &RequestContext,
    ) -> Result<&Deployment>;

    /// 执行请求 (带重试和 fallback)
    pub async fn completion(
        &self,
        model_name: &str,
        request: ChatCompletionRequest,
        context: RequestContext,
    ) -> Result<ChatCompletionResponse>;

    /// 流式请求
    pub async fn completion_stream(
        &self,
        model_name: &str,
        request: ChatCompletionRequest,
        context: RequestContext,
    ) -> Result<ChatCompletionStream>;

    // ========== 状态更新 ==========

    /// 记录成功
    pub fn record_success(&self, deployment_id: &str, tokens: u64, latency_us: u64);

    /// 记录失败
    pub fn record_failure(&self, deployment_id: &str, error: &ProviderError);

    // ========== 查询 ==========

    /// 获取 deployment 列表
    pub fn get_deployments(&self, model_name: &str) -> Vec<&Deployment>;

    /// 获取健康的 deployments
    pub fn get_healthy_deployments(&self, model_name: &str) -> Vec<&Deployment>;

    /// 获取 fallback models
    pub fn get_fallbacks(&self, model_name: &str, error: &ProviderError) -> Vec<ModelName>;
}
```

### 2.3 路由选择算法

```rust
impl Router {
    /// 核心: 选择最佳 deployment
    async fn select_deployment(
        &self,
        model_name: &str,
        context: &RequestContext,
    ) -> Result<&Deployment> {
        // 1. 获取该 model 的所有 deployments
        let deployment_ids = self.model_index.get(model_name)
            .ok_or(RouterError::ModelNotFound(model_name.to_string()))?;

        // 2. 过滤: 健康 + 非冷却 + 未超限
        let now = current_timestamp();
        let candidates: Vec<&Deployment> = deployment_ids.iter()
            .filter_map(|id| self.deployments.get(id))
            .filter(|d| {
                // 非冷却
                d.state.cooldown_until.load(Relaxed) < now &&
                // 健康
                d.state.health.load(Relaxed) < 3 &&
                // 未超并发限制
                self.check_parallel_limit(d) &&
                // 未超 RPM/TPM (如果启用预检查)
                self.check_rate_limit(d, context)
            })
            .collect();

        if candidates.is_empty() {
            return Err(RouterError::NoAvailableDeployment(model_name.to_string()));
        }

        // 3. 按策略选择
        let selected = match self.strategy {
            RoutingStrategy::SimpleShuffle => self.select_weighted_random(&candidates),
            RoutingStrategy::LeastBusy => self.select_least_busy(&candidates),
            RoutingStrategy::UsageBased => self.select_lowest_usage(&candidates),
            RoutingStrategy::LatencyBased => self.select_lowest_latency(&candidates),
            RoutingStrategy::CostBased => self.select_lowest_cost(&candidates, context),
            RoutingStrategy::RateLimitAware => self.select_rate_limit_aware(&candidates),
            RoutingStrategy::RoundRobin => self.select_round_robin(&candidates),
        };

        // 4. 增加活跃请求计数
        selected.state.active_requests.fetch_add(1, Relaxed);

        Ok(selected)
    }

    /// 加权随机选择
    fn select_weighted_random<'a>(&self, candidates: &[&'a Deployment]) -> &'a Deployment {
        let total_weight: u32 = candidates.iter().map(|d| d.config.weight).sum();
        let mut rng = thread_rng();
        let mut point = rng.gen_range(0..total_weight);

        for d in candidates {
            if point < d.config.weight {
                return d;
            }
            point -= d.config.weight;
        }
        candidates[0]
    }

    /// 最少并发选择
    fn select_least_busy<'a>(&self, candidates: &[&'a Deployment]) -> &'a Deployment {
        candidates.iter()
            .min_by_key(|d| d.state.active_requests.load(Relaxed))
            .unwrap()
    }

    /// 最低使用率选择
    fn select_lowest_usage<'a>(&self, candidates: &[&'a Deployment]) -> &'a Deployment {
        candidates.iter()
            .min_by_key(|d| {
                let tpm = d.state.tpm_current.load(Relaxed);
                let tpm_limit = d.config.tpm_limit.unwrap_or(u64::MAX);
                // 计算使用率百分比
                (tpm * 100) / tpm_limit.max(1)
            })
            .unwrap()
    }

    /// 最低延迟选择
    fn select_lowest_latency<'a>(&self, candidates: &[&'a Deployment]) -> &'a Deployment {
        candidates.iter()
            .min_by_key(|d| d.state.avg_latency_us.load(Relaxed))
            .unwrap()
    }
}
```

### 2.4 Cooldown 机制

```rust
impl Router {
    /// 检查是否需要进入 cooldown
    fn check_cooldown(&self, deployment: &Deployment, error: &ProviderError) {
        let should_cooldown = match error {
            // 429 Rate Limit -> 立即 cooldown
            ProviderError::RateLimit { .. } => true,

            // 不可重试错误 -> cooldown
            ProviderError::Authentication { .. } |
            ProviderError::ModelNotFound { .. } => true,

            // 其他错误 -> 检查失败率
            _ => {
                let fails = deployment.state.fails_this_minute.fetch_add(1, Relaxed) + 1;
                fails >= self.config.allowed_fails
            }
        };

        if should_cooldown {
            let cooldown_until = current_timestamp() + self.config.cooldown_time;
            deployment.state.cooldown_until.store(cooldown_until, Relaxed);
            deployment.state.health.store(4, Relaxed); // 4 = cooldown
        }
    }

    /// 检查 cooldown 是否结束
    fn is_in_cooldown(&self, deployment: &Deployment) -> bool {
        deployment.state.cooldown_until.load(Relaxed) > current_timestamp()
    }
}
```

### 2.5 Retry + Fallback 流程

```rust
impl Router {
    /// 带重试和 fallback 的请求
    pub async fn completion(
        &self,
        model_name: &str,
        request: ChatCompletionRequest,
        context: RequestContext,
    ) -> Result<ChatCompletionResponse> {
        let mut models_to_try = vec![model_name.to_string()];
        let mut last_error = None;
        let mut fallback_count = 0;

        while let Some(current_model) = models_to_try.first() {
            // 尝试当前模型 (带重试)
            match self.try_with_retries(current_model, &request, &context).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e.clone());

                    // 获取 fallback
                    if fallback_count < self.config.max_fallbacks {
                        let fallbacks = self.get_fallbacks(current_model, &e);
                        for fb in fallbacks {
                            if !models_to_try.contains(&fb) {
                                models_to_try.push(fb);
                            }
                        }
                        fallback_count += 1;
                    }
                }
            }
            models_to_try.remove(0);
        }

        Err(last_error.unwrap_or(RouterError::NoAvailableDeployment(model_name.to_string()).into()))
    }

    /// 带重试的单模型请求
    async fn try_with_retries(
        &self,
        model_name: &str,
        request: &ChatCompletionRequest,
        context: &RequestContext,
    ) -> Result<ChatCompletionResponse> {
        let mut last_error = None;

        for attempt in 0..=self.config.num_retries {
            // 选择 deployment
            let deployment = match self.select_deployment(model_name, context).await {
                Ok(d) => d,
                Err(e) => {
                    last_error = Some(e.into());
                    continue;
                }
            };

            // 执行请求
            let start = Instant::now();
            let result = deployment.provider
                .chat_completion(request.clone(), context.clone())
                .await;
            let latency_us = start.elapsed().as_micros() as u64;

            // 减少活跃请求计数
            deployment.state.active_requests.fetch_sub(1, Relaxed);

            match result {
                Ok(response) => {
                    // 记录成功
                    let tokens = response.usage.as_ref()
                        .map(|u| u.total_tokens as u64)
                        .unwrap_or(0);
                    self.record_success(&deployment.id, tokens, latency_us);
                    return Ok(response);
                }
                Err(e) => {
                    // 记录失败
                    self.record_failure(&deployment.id, &e);

                    // 检查是否可重试
                    if !e.is_retryable() || attempt == self.config.num_retries {
                        last_error = Some(e.into());
                        break;
                    }

                    // 等待后重试
                    if self.config.retry_after > 0 {
                        tokio::time::sleep(Duration::from_secs(self.config.retry_after)).await;
                    }
                    last_error = Some(e.into());
                }
            }
        }

        Err(last_error.unwrap())
    }
}
```

---

## 3. 性能优化设计

### 3.1 Lock-Free 数据结构

| 组件 | Python | Rust | 性能提升 |
|------|--------|------|----------|
| Deployment 存储 | `dict` + threading.Lock | `DashMap` | ~10x 并发 |
| 计数器 | `threading.Lock` + int | `AtomicU64` | ~100x |
| 状态 | dict + Lock | 独立 Atomic 字段 | 无锁 |

### 3.2 静态派发 vs 动态派发

```rust
// Rust: 静态派发 (Provider enum)
match deployment.provider {
    Provider::OpenAI(p) => p.chat_completion(req, ctx).await,
    Provider::Anthropic(p) => p.chat_completion(req, ctx).await,
    // ... 编译时确定，无 vtable 开销
}

// Python: 动态派发
provider.chat_completion(req)  # 运行时查找方法
```

### 3.3 零拷贝 / 最小分配

```rust
// Request context 共享
let context = Arc::new(context);

// Deployment 引用，不克隆
let deployment: &Deployment = self.select_deployment(model).await?;

// Provider 不克隆，直接使用引用
deployment.provider.chat_completion(request, context).await
```

### 3.4 分钟窗口重置 (后台任务)

```rust
impl Router {
    /// 启动后台重置任务
    pub fn start_minute_reset_task(&self) {
        let deployments = self.deployments.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let now = current_timestamp();

                for mut entry in deployments.iter_mut() {
                    let state = &entry.state;
                    state.tpm_current.store(0, Relaxed);
                    state.rpm_current.store(0, Relaxed);
                    state.fails_this_minute.store(0, Relaxed);
                    state.minute_reset_at.store(now, Relaxed);
                }
            }
        });
    }
}
```

---

## 4. 配置格式

### 4.1 YAML 配置

```yaml
router:
  # 路由策略
  routing_strategy: simple_shuffle  # simple_shuffle | least_busy | usage_based | latency_based | cost_based | rate_limit_aware

  # 重试配置
  num_retries: 3
  retry_after: 0  # 秒

  # Cooldown 配置
  allowed_fails: 3
  cooldown_time: 5  # 秒

  # 超时
  timeout: 60  # 秒

  # Fallback 配置
  max_fallbacks: 5

  # 预检查
  enable_pre_call_checks: true

  # Fallbacks
  fallbacks:
    general:
      gpt-4: [gpt-4-turbo, gpt-3.5-turbo]
    context_window:
      gpt-4: [gpt-4-32k, claude-3-opus]
    content_policy:
      gpt-4: [claude-3-opus]

# Model List
model_list:
  - model_name: gpt-4
    deployments:
      - id: openai-gpt4-primary
        provider: openai
        model: gpt-4-turbo
        api_key: ${OPENAI_API_KEY}
        rpm: 500
        tpm: 100000
        max_parallel_requests: 10
        weight: 2
        priority: 0
        tags: [production, fast]

      - id: azure-gpt4-backup
        provider: azure
        model: gpt-4
        api_key: ${AZURE_API_KEY}
        api_base: ${AZURE_API_BASE}
        rpm: 200
        tpm: 50000
        weight: 1
        priority: 1
        tags: [backup]

  - model_name: gpt-3.5-turbo
    deployments:
      - id: openai-gpt35
        provider: openai
        model: gpt-3.5-turbo
        api_key: ${OPENAI_API_KEY}
        rpm: 1000
        tpm: 200000
```

---

## 5. 文件结构

```
src/core/router/
├── mod.rs              # 模块入口，重新导出
├── router.rs           # Router 主结构 (NEW)
├── deployment.rs       # Deployment 结构 (NEW)
├── strategy.rs         # 路由策略 (重构，简化)
├── fallback.rs         # Fallback 逻辑 (NEW，从 load_balancer.rs 提取)
├── cooldown.rs         # Cooldown 逻辑 (NEW，集成 CircuitBreaker)
├── config.rs           # 配置解析 (NEW)
├── metrics.rs          # 统计 (保留)
├── error.rs            # 错误类型 (NEW)
└── tests/              # 测试
    ├── mod.rs
    ├── router_test.rs
    ├── strategy_test.rs
    └── fallback_test.rs

# 删除
├── router_v2.rs        # 删除
├── load_balancer.rs    # 功能合并到 router.rs
├── health.rs           # 功能合并到 deployment.rs
```

---

## 6. 实现步骤

### Phase 1: 核心结构 (Day 1)

1. 创建 `deployment.rs`:
   - `Deployment` 结构
   - `DeploymentConfig` 结构
   - `DeploymentState` (全 atomic)

2. 创建 `router.rs`:
   - `Router` 结构
   - `RouterConfig` 结构
   - Deployment 管理方法

### Phase 2: 路由策略 (Day 1)

1. 重构 `strategy.rs`:
   - 简化 `RoutingStrategy` enum
   - 实现 6 种策略的选择方法

2. 实现 `select_deployment()` 核心方法

### Phase 3: Cooldown + Retry + Fallback (Day 2)

1. 创建 `cooldown.rs`:
   - Cooldown 检测逻辑
   - 自动恢复

2. 创建 `fallback.rs`:
   - `FallbackConfig`
   - 按错误类型获取 fallback

3. 实现 `completion()` 完整流程

### Phase 4: 集成 (Day 2)

1. 更新 `server/mod.rs`:
   - 使用新 Router

2. 更新 `completion.rs`:
   - DefaultRouter 使用新 Router

3. 创建 `config.rs`:
   - YAML 配置解析

### Phase 5: 测试 + 清理 (Day 3)

1. 单元测试
2. 集成测试
3. 删除废弃文件
4. 更新文档

---

## 7. 性能基准测试结果 (实测)

> 以下数据来自 `cargo bench` 实际测试，运行于 Apple Silicon

### 7.1 单次操作性能

| 操作 | 实测时间 | 说明 |
|------|---------|------|
| Router 创建 | **39.4 ns** | 创建空路由器 |
| 添加 Deployment | **1.04 µs** | 插入单个 deployment |
| 别名解析 | **31.9 ns** | model name 别名查找 |
| 记录成功 | **47.3 ns** | 原子操作更新计数器 |
| 记录失败 | **65.5 ns** | 原子操作更新失败计数 |

### 7.2 路由策略性能 (10 deployments)

| 策略 | 实测时间 | 说明 |
|------|---------|------|
| **RoundRobin** | 1.24 µs | 最快 - 简单计数器 |
| **LatencyBased** | 1.81 µs | 需比较延迟 |
| **SimpleShuffle** | 1.85 µs | 加权随机 |
| **LeastBusy** | 2.04 µs | 需比较活跃请求数 |

### 7.3 获取健康 Deployment (按数量)

| Deployment 数量 | 实测时间 | 吞吐量 |
|----------------|---------|--------|
| 1 | 130 ns | ~7.7M ops/s |
| 5 | 388 ns | ~2.6M ops/s |
| 10 | 694 ns | ~1.4M ops/s |
| 50 | 3.2 µs | ~312K ops/s |
| 100 | 6.3 µs | ~159K ops/s |

### 7.4 并发性能 (Lock-Free 操作)

| 并发任务数 | 实测时间 | 吞吐量 |
|-----------|---------|--------|
| 10 | 37.3 µs | ~268K ops/s |
| 50 | 97.7 µs | ~512K ops/s |
| 100 | 172 µs | ~581K ops/s |
| 500 | 721 µs | **~693K ops/s** |

### 7.5 与 Python LiteLLM 预估对比

| 指标 | Python (预估) | Rust (实测) | 提升 |
|------|--------------|-------------|------|
| Route selection | ~100μs | **1.2-2.0 µs** | ~50-80x |
| Record success/fail | ~10μs | **47-65 ns** | ~150-200x |
| Concurrent throughput | ~1K ops/s | **~700K ops/s** | ~700x |
| Lock contention | High (GIL) | None | ∞ |
| GC pauses | Yes | No | - |

### 7.6 运行基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定测试组
cargo bench -- unified_router      # Router 操作
cargo bench -- concurrent_router   # 并发性能
cargo bench -- cache_operations    # 缓存测试

# 快速运行（跳过图表生成）
cargo bench -- --noplot
```

测试报告保存在 `target/criterion/` 目录。

---

## 8. API 兼容性

### 与 Python LiteLLM 兼容

```rust
// Python: router.completion(model="gpt-4", messages=[...])
// Rust:
let response = router.completion("gpt-4", request, context).await?;

// Python: router.acompletion(model="gpt-4", messages=[...])
// Rust: 同上，Rust 天然 async

// Python: router.get_available_deployment(model="gpt-4")
// Rust:
let deployment = router.get_available_deployment("gpt-4", &context).await?;
```

### 与现有代码兼容

```rust
// 保持 completion() 函数的 API
pub async fn completion(
    model: &str,
    messages: Vec<Message>,
    options: Option<CompletionOptions>,
) -> Result<CompletionResponse> {
    let router = get_global_router().await;
    router.completion(model, request, context).await
}
```

---

## 9. 总结

### 功能完整性

✅ Model List / Deployments
✅ 6 种 Routing Strategies
✅ 4 种 Fallback 类型
✅ Cooldown 机制
✅ Rate Limiting (TPM/RPM)
✅ Retries
✅ Health Tracking
✅ Pre-call Checks
✅ Concurrent Limit
✅ Weighted Selection
✅ Tag Filtering

### 性能优势

- **Lock-Free**: DashMap + Atomics
- **静态派发**: Provider enum
- **零 GC**: Rust 内存管理
- **最小分配**: 引用 + Arc 共享

### 代码简化

- 3 个 Router → 1 个
- 删除 router_v2.rs, load_balancer.rs (功能合并)
- 清晰的职责划分
