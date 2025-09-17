//! 中间件系统 trait 定义
//!
//! 提供可组合的中间件架构，支持认证、cache、重试etc横切关注点

use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

/// 中间件核心 trait
///
/// 所有中间件都必须implementation此 trait
#[async_trait]
pub trait Middleware<Req, Resp>: Send + Sync {
    /// Error
    type Error: std::error::Error + Send + Sync + 'static;

    /// Handle
    ///
    /// # parameter
    /// Request
    /// Handle
    ///
    /// # Returns
    /// Handle
    async fn process(
        &self,
        request: Req,
        next: Box<dyn MiddlewareNext<Req, Resp>>,
    ) -> Result<Resp, Self::Error>;
}

/// Handle
#[async_trait]
pub trait MiddlewareNext<Req, Resp>: Send + Sync {
    /// Handle
    async fn call(&self, request: Req) -> Result<Resp, Box<dyn std::error::Error + Send + Sync>>;
}

/// 中间件链/栈
pub struct MiddlewareStack<Req, Resp> {
    middlewares:
        Vec<Box<dyn Middleware<Req, Resp, Error = Box<dyn std::error::Error + Send + Sync>>>>,
}

impl<Req, Resp> MiddlewareStack<Req, Resp>
where
    Req: Clone + Send + Sync + 'static,
    Resp: Send + Sync + 'static,
{
    /// Create
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// 添加中间件
    pub fn add<M>(self, _middleware: M) -> Self
    where
        M: Middleware<Req, Resp> + 'static,
    {
        // TODO: Fix middleware wrapper type constraints
        // let boxed = Box::new(MiddlewareWrapper(middleware));
        // self.middlewares.push(boxed);
        self
    }

    /// 执行中间件链
    pub async fn execute<F, Fut>(
        &self,
        request: Req,
        final_handler: F,
    ) -> Result<Resp, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Resp, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + Sync
            + 'static,
    {
        let handler = Box::new(FinalHandler::new(final_handler));
        self.execute_chain(request, 0, handler).await
    }

    /// 递归执行中间件链
    fn execute_chain(
        &self,
        request: Req,
        index: usize,
        final_handler: Box<dyn MiddlewareNext<Req, Resp>>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Resp, Box<dyn std::error::Error + Send + Sync>>> + Send + '_,
        >,
    > {
        Box::pin(async move {
            if index >= self.middlewares.len() {
                // Handle
                final_handler.call(request).await
            } else {
                // Create
                let _next = Box::new(NextHandler {
                    stack: self,
                    index: index + 1,
                    final_handler,
                    request: request.clone(),
                });

                // TODO: Fix middleware execution with proper type constraints
                // self.middlewares[index].process(request, next).await
                Err(Box::new(std::io::Error::other(
                    "Middleware system temporarily disabled",
                ))
                    as Box<dyn std::error::Error + Send + Sync>)
            }
        })
    }
}

impl<Req, Resp> Default for MiddlewareStack<Req, Resp>
where
    Req: Clone + Send + Sync + 'static,
    Resp: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// 中间件包装器，用于类型擦除
struct MiddlewareWrapper<M>(M);

#[async_trait]
impl<M, Req, Resp> Middleware<Req, Resp> for MiddlewareWrapper<M>
where
    M: Middleware<Req, Resp> + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    Req: Send + Sync + 'static,
    Resp: Send + Sync + 'static,
{
    type Error = M::Error;

    async fn process(
        &self,
        request: Req,
        next: Box<dyn MiddlewareNext<Req, Resp>>,
    ) -> Result<Resp, Self::Error> {
        self.0.process(request, next).await
    }
}

/// Handle
struct FinalHandler<F, Fut, Req, Resp> {
    handler: Option<F>,
    _phantom: std::marker::PhantomData<(Fut, Req, Resp)>,
}

impl<F, Fut, Req, Resp> FinalHandler<F, Fut, Req, Resp> {
    fn new(handler: F) -> Self {
        Self {
            handler: Some(handler),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<F, Fut, Req, Resp> MiddlewareNext<Req, Resp> for FinalHandler<F, Fut, Req, Resp>
where
    F: FnOnce(Req) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Resp, Box<dyn std::error::Error + Send + Sync>>> + Send + Sync,
    Req: Send + Sync,
    Resp: Send + Sync,
{
    async fn call(&self, _request: Req) -> Result<Resp, Box<dyn std::error::Error + Send + Sync>> {
        // 这里有个问题，FnOnce 只能call一次，但 trait 方法可能被多次call
        // 实际implementation中需要更复杂的设计
        todo!("Implement proper FnOnce handling")
    }
}

/// Handle
struct NextHandler<'a, Req, Resp> {
    stack: &'a MiddlewareStack<Req, Resp>,
    index: usize,
    final_handler: Box<dyn MiddlewareNext<Req, Resp>>,
    request: Req,
}

#[async_trait]
impl<'a, Req, Resp> MiddlewareNext<Req, Resp> for NextHandler<'a, Req, Resp>
where
    Req: Clone + Send + Sync + 'static,
    Resp: Send + Sync + 'static,
{
    async fn call(&self, _request: Req) -> Result<Resp, Box<dyn std::error::Error + Send + Sync>> {
        // 这里也需要重新设计，因为生命周期的问题
        todo!("Implement proper next handler")
    }
}

/// Error
#[derive(Debug, thiserror::Error)]
pub enum MiddlewareError {
    #[error("Middleware chain execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid middleware configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Middleware timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Other middleware error: {0}")]
    Other(String),
}
