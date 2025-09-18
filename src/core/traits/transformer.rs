//! Transformer trait definitions
//!
//! Provides unified interface for request/response format conversion between different providers

use futures::Stream;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Core transformer trait
///
/// Provides capability to convert from one format to another
pub trait Transform<From, To>: Send + Sync {
    /// Error
    type Error: std::error::Error + Send + Sync + 'static;

    /// Single item transformation
    fn transform(input: From) -> Result<To, Self::Error>;

    /// Batch transformation
    fn transform_batch(inputs: Vec<From>) -> Result<Vec<To>, Self::Error> {
        inputs.into_iter().map(Self::transform).collect()
    }

    /// Stream transformation
    fn transform_stream<S>(stream: S) -> TransformStream<S, Self>
    where
        S: Stream<Item = From> + Send,
        Self: Sized,
    {
        TransformStream::new(stream, PhantomData)
    }
}

/// Bidirectional transformer trait
///
/// Transformer that supports bidirectional conversion
pub trait BidirectionalTransform<A, B>: Transform<A, B> + Transform<B, A> {
    /// A to B conversion
    fn forward(input: A) -> Result<B, <Self as Transform<A, B>>::Error> {
        <Self as Transform<A, B>>::transform(input)
    }

    /// B to A conversion  
    fn backward(input: B) -> Result<A, <Self as Transform<B, A>>::Error> {
        <Self as Transform<B, A>>::transform(input)
    }
}

/// Stream transformation wrapper
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

/// Transformer registry
pub struct TransformerRegistry {
    // Here we can store mappings of different type transformers
    // Simplified implementation for now
}

impl TransformerRegistry {
    pub fn new() -> Self {
        Self {}
    }

    pub fn register_transformer<T>(&mut self, _transformer: T)
    where
        T: Transform<(), ()> + 'static,
    {
        // In actual implementation, transformers would be stored
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
