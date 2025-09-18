//! Performance benchmarks for litellm-rs
//!
//! This module contains comprehensive benchmarks to measure the performance
//! of various components in the litellm-rs system.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use litellm_rs::core::cache_manager::{CacheConfig, CacheManager};
use litellm_rs::core::models::openai::*;
use litellm_rs::core::router::load_balancer::LoadBalancer;
use litellm_rs::core::router::strategy::RoutingStrategy;
use std::hint::black_box;

use litellm_rs::utils::string_pool::{StringPool, intern_string};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark cache operations
fn bench_cache_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cache_operations");

    // Test different cache sizes
    for cache_size in [100, 1000, 10000].iter() {
        let config = CacheConfig {
            max_entries: *cache_size,
            default_ttl: Duration::from_secs(3600),
            enable_semantic: false,
            similarity_threshold: 0.95,
            min_prompt_length: 10,
            enable_compression: false,
        };

        group.bench_with_input(
            BenchmarkId::new("cache_get", cache_size),
            cache_size,
            |b, &_size| {
                let cache = rt.block_on(async { CacheManager::new(config.clone()).unwrap() });
                let key = litellm_rs::core::cache_manager::CacheKey {
                    model: intern_string("gpt-4"),
                    request_hash: 12345,
                    user_id: None,
                };

                b.iter(|| rt.block_on(async { black_box(cache.get(&key).await.unwrap()) }));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("cache_put", cache_size),
            cache_size,
            |b, &_size| {
                let cache = rt.block_on(async { CacheManager::new(config.clone()).unwrap() });

                b.iter(|| {
                    let key = litellm_rs::core::cache_manager::CacheKey {
                        model: intern_string("gpt-4"),
                        request_hash: rand::random::<u64>(),
                        user_id: None,
                    };
                    let response = ChatCompletionResponse {
                        id: "test".to_string(),
                        object: "chat.completion".to_string(),
                        created: 1234567890,
                        model: "gpt-4".to_string(),
                        choices: vec![],
                        usage: None,
                        system_fingerprint: None,
                    };

                    rt.block_on(async {
                        cache.put(key, response).await.unwrap();
                        black_box(())
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark string pool operations
fn bench_string_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_pool");

    // Test string interning performance
    group.bench_function("intern_new_strings", |b| {
        let pool = StringPool::new();
        let mut counter = 0;

        b.iter(|| {
            counter += 1;
            let s = format!("test_string_{}", counter);
            black_box(pool.intern(&s))
        });
    });

    group.bench_function("intern_existing_strings", |b| {
        let pool = StringPool::new();
        // Pre-populate with some strings
        for i in 0..100 {
            pool.intern(&format!("test_string_{}", i));
        }

        b.iter(|| {
            let s = format!("test_string_{}", rand::random::<u8>() % 100);
            black_box(pool.intern(&s))
        });
    });

    group.bench_function("global_intern", |b| {
        b.iter(|| {
            let s = format!("global_test_{}", rand::random::<u32>());
            black_box(intern_string(&s))
        });
    });

    group.finish();
}

/// Benchmark load balancer operations
fn bench_load_balancer(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_balancer");

    group.bench_function("provider_creation", |b| {
        let rt = Runtime::new().unwrap();

        b.iter(|| {
            rt.block_on(async {
                black_box(
                    LoadBalancer::new(RoutingStrategy::RoundRobin)
                        .await
                        .unwrap(),
                )
            })
        });
    });

    group.finish();
}

/// Benchmark serialization/deserialization
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    group.throughput(Throughput::Elements(1));

    let request = ChatCompletionRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text(
                    "You are a helpful assistant.".to_string(),
                )),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello, how are you?".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(150),
        max_completion_tokens: None,
        top_p: Some(1.0),
        n: Some(1),
        stream: Some(false),
        stream_options: None,
        stop: None,
        presence_penalty: Some(0.0),
        frequency_penalty: Some(0.0),
        logit_bias: None,
        user: None,
        functions: None,
        function_call: None,
        tools: None,
        tool_choice: None,
        response_format: None,
        seed: None,
        logprobs: None,
        top_logprobs: None,
        modalities: None,
        audio: None,
    };

    group.bench_function("serialize_request", |b| {
        b.iter(|| black_box(serde_json::to_string(&request).unwrap()));
    });

    let json_str = serde_json::to_string(&request).unwrap();
    group.bench_function("deserialize_request", |b| {
        b.iter(|| black_box(serde_json::from_str::<ChatCompletionRequest>(&json_str).unwrap()));
    });

    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_operations");

    // Test concurrent cache operations
    for num_tasks in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_cache_ops", num_tasks),
            num_tasks,
            |b, &num_tasks| {
                let config = CacheConfig::default();
                let cache = rt.block_on(async { Arc::new(CacheManager::new(config).unwrap()) });

                b.iter(|| {
                    let cache = cache.clone();
                    rt.block_on(async move {
                        let mut handles = Vec::new();

                        for i in 0..num_tasks {
                            let cache = cache.clone();
                            let handle = tokio::spawn(async move {
                                let key = litellm_rs::core::cache_manager::CacheKey {
                                    model: intern_string("gpt-4"),
                                    request_hash: i as u64,
                                    user_id: None,
                                };
                                cache.get(&key).await.unwrap()
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            black_box(handle.await.unwrap());
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // Test memory allocation patterns
    group.bench_function("string_allocations", |b| {
        b.iter(|| {
            let mut strings = Vec::new();
            for i in 0..1000 {
                strings.push(format!("test_string_{}", i));
            }
            black_box(strings)
        });
    });

    group.bench_function("arc_allocations", |b| {
        b.iter(|| {
            let mut arcs = Vec::new();
            for i in 0..1000 {
                arcs.push(Arc::new(format!("test_string_{}", i)));
            }
            black_box(arcs)
        });
    });

    group.bench_function("interned_strings", |b| {
        b.iter(|| {
            let mut strings = Vec::new();
            for i in 0..1000 {
                strings.push(intern_string(&format!("test_string_{}", i)));
            }
            black_box(strings)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_operations,
    bench_string_pool,
    bench_load_balancer,
    bench_serialization,
    bench_concurrent_operations,
    bench_memory_usage
);

criterion_main!(benches);
