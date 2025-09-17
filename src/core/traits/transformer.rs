//! 转换器 trait 定义
//!
//! 提供不同 provider 之间请求/响应format转换的统一接口

use futures::Stream;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

/// 转换器核心 trait
///
/// 提供从一种format转换到另一种format的能力
pub trait Transform<From, To>: Send + Sync {
    /// Error
    type Error: std::error::Error + Send + Sync + 'static;

    /// 单次转换
    fn transform(input: From) -> Result<To, Self::Error>;

    /// 批量转换
    fn transform_batch(inputs: Vec<From>) -> Result<Vec<To>, Self::Error> {
        inputs.into_iter().map(Self::transform).collect()
    }

    /// 流式转换
    fn transform_stream<S>(stream: S) -> TransformStream<S, Self>
    where
        S: Stream<Item = From> + Send,
        Self: Sized,
    {
        TransformStream::new(stream, PhantomData)
    }
}

/// 双向转换器 trait
///
/// 支持双向转换的转换器
pub trait BidirectionalTransform<A, B>: Transform<A, B> + Transform<B, A> {
    /// A 到 B 的转换
    fn forward(input: A) -> Result<B, <Self as Transform<A, B>>::Error> {
        <Self as Transform<A, B>>::transform(input)
    }

    /// B 到 A 的转换  
    fn backward(input: B) -> Result<A, <Self as Transform<B, A>>::Error> {
        <Self as Transform<B, A>>::transform(input)
    }
}

/// 流式转换包装器
pub struct TransformStream<S, T> {
    stream: S,
    _phantom: PhantomData<T>,
}

impl<S, T> TransformStream<S, T> {
    pub fn new(stream: S, _phantom: PhantomData<T>) -> Self {
        Self { stream, _phantom }
    }
}

impl<S, T> Stream for TransformStream<S, T>
where
    S: Stream + Unpin,
    T: Send + Sync,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = unsafe { self.get_unchecked_mut() };
        Pin::new(&mut this.stream).poll_next(cx)
    }
}

/// 转换器注册表
pub struct TransformerRegistry {
    // 这里可以存储不同类型转换器的映射
    // 暂时简化implementation
}

impl TransformerRegistry {
    pub fn new() -> Self {
        Self {}
    }

    pub fn register_transformer<T>(&mut self, _transformer: T)
    where
        T: Transform<(), ()> + 'static,
    {
        // 实际implementation中会存储转换器
        todo!("Implement transformer registration")
    }
}

impl Default for TransformerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Error
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    #[error("Conversion failed: {0}")]
    ConversionFailed(String),

    #[error("Unsupported conversion from {from} to {to}")]
    UnsupportedConversion { from: String, to: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid value for field {field}: {value}")]
    InvalidValue { field: String, value: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other transform error: {0}")]
    Other(String),
}

impl TransformError {
    pub fn conversion_failed(msg: impl Into<String>) -> Self {
        Self::ConversionFailed(msg.into())
    }

    pub fn unsupported_conversion(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::UnsupportedConversion {
            from: from.into(),
            to: to.into(),
        }
    }

    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    pub fn invalid_value(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::InvalidValue {
            field: field.into(),
            value: value.into(),
        }
    }
}
