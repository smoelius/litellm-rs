//! Server configuration types

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Listen address
    pub host: String,
    /// Listen port
    pub port: u16,
    /// Worker thread count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workers: Option<usize>,
    /// Maximum connections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<usize>,
    /// Request timeout
    #[serde(with = "super::duration_serde")]
    pub timeout: Duration,
    /// TLS configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
    /// Enabled features
    #[serde(default)]
    pub features: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: None,
            max_connections: None,
            timeout: Duration::from_secs(30),
            tls: None,
            features: Vec::new(),
        }
    }
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate file path
    pub cert_file: String,
    /// Private key file path
    pub key_file: String,
    /// Enable HTTP/2
    #[serde(default)]
    pub http2: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ServerConfig Default Tests ====================

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(config.workers.is_none());
        assert!(config.max_connections.is_none());
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.tls.is_none());
        assert!(config.features.is_empty());
    }

    #[test]
    fn test_server_config_custom_values() {
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            workers: Some(4),
            max_connections: Some(1000),
            timeout: Duration::from_secs(60),
            tls: None,
            features: vec!["feature1".to_string(), "feature2".to_string()],
        };

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert_eq!(config.workers, Some(4));
        assert_eq!(config.max_connections, Some(1000));
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.features.len(), 2);
    }

    // ==================== ServerConfig Serialization Tests ====================

    #[test]
    fn test_server_config_serialize_default() {
        let config = ServerConfig::default();
        let json = serde_json::to_string(&config).unwrap();

        // Optional None fields should be skipped
        assert!(!json.contains("workers"));
        assert!(!json.contains("max_connections"));
        assert!(!json.contains("tls"));

        // Required fields should be present
        assert!(json.contains("host"));
        assert!(json.contains("port"));
        assert!(json.contains("timeout"));
    }

    #[test]
    fn test_server_config_serialize_with_optional_fields() {
        let config = ServerConfig {
            host: "localhost".to_string(),
            port: 9000,
            workers: Some(8),
            max_connections: Some(500),
            timeout: Duration::from_secs(45),
            tls: Some(TlsConfig {
                cert_file: "/path/to/cert.pem".to_string(),
                key_file: "/path/to/key.pem".to_string(),
                http2: true,
            }),
            features: vec!["auth".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("workers"));
        assert!(json.contains("max_connections"));
        assert!(json.contains("tls"));
        assert!(json.contains("cert_file"));
    }

    #[test]
    fn test_server_config_deserialize_minimal() {
        let json = r#"{"host":"0.0.0.0","port":8080,"timeout":30}"#;
        let config: ServerConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.features.is_empty()); // Default empty vec
    }

    #[test]
    fn test_server_config_deserialize_full() {
        let json = r#"{
            "host": "192.168.1.1",
            "port": 443,
            "workers": 16,
            "max_connections": 10000,
            "timeout": 120,
            "tls": {
                "cert_file": "/etc/ssl/cert.pem",
                "key_file": "/etc/ssl/key.pem",
                "http2": true
            },
            "features": ["metrics", "tracing"]
        }"#;

        let config: ServerConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.host, "192.168.1.1");
        assert_eq!(config.port, 443);
        assert_eq!(config.workers, Some(16));
        assert_eq!(config.max_connections, Some(10000));
        assert_eq!(config.timeout, Duration::from_secs(120));
        assert!(config.tls.is_some());

        let tls = config.tls.unwrap();
        assert_eq!(tls.cert_file, "/etc/ssl/cert.pem");
        assert_eq!(tls.key_file, "/etc/ssl/key.pem");
        assert!(tls.http2);

        assert_eq!(config.features, vec!["metrics", "tracing"]);
    }

    #[test]
    fn test_server_config_roundtrip() {
        let original = ServerConfig {
            host: "api.example.com".to_string(),
            port: 8443,
            workers: Some(4),
            max_connections: Some(2000),
            timeout: Duration::from_secs(90),
            tls: Some(TlsConfig {
                cert_file: "cert.pem".to_string(),
                key_file: "key.pem".to_string(),
                http2: false,
            }),
            features: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ServerConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original.host, deserialized.host);
        assert_eq!(original.port, deserialized.port);
        assert_eq!(original.workers, deserialized.workers);
        assert_eq!(original.max_connections, deserialized.max_connections);
        assert_eq!(original.timeout, deserialized.timeout);
        assert_eq!(original.features, deserialized.features);
    }

    // ==================== ServerConfig Clone Tests ====================

    #[test]
    fn test_server_config_clone() {
        let original = ServerConfig {
            host: "test".to_string(),
            port: 1234,
            workers: Some(2),
            max_connections: None,
            timeout: Duration::from_secs(10),
            tls: None,
            features: vec!["x".to_string()],
        };

        let cloned = original.clone();
        assert_eq!(original.host, cloned.host);
        assert_eq!(original.port, cloned.port);
        assert_eq!(original.workers, cloned.workers);
    }

    // ==================== TlsConfig Tests ====================

    #[test]
    fn test_tls_config_basic() {
        let tls = TlsConfig {
            cert_file: "/path/to/cert.pem".to_string(),
            key_file: "/path/to/key.pem".to_string(),
            http2: false,
        };

        assert_eq!(tls.cert_file, "/path/to/cert.pem");
        assert_eq!(tls.key_file, "/path/to/key.pem");
        assert!(!tls.http2);
    }

    #[test]
    fn test_tls_config_with_http2() {
        let tls = TlsConfig {
            cert_file: "cert.pem".to_string(),
            key_file: "key.pem".to_string(),
            http2: true,
        };

        assert!(tls.http2);
    }

    #[test]
    fn test_tls_config_serialize() {
        let tls = TlsConfig {
            cert_file: "server.crt".to_string(),
            key_file: "server.key".to_string(),
            http2: true,
        };

        let json = serde_json::to_string(&tls).unwrap();
        assert!(json.contains("server.crt"));
        assert!(json.contains("server.key"));
        assert!(json.contains("http2"));
    }

    #[test]
    fn test_tls_config_deserialize_with_default_http2() {
        // http2 should default to false when not specified
        let json = r#"{"cert_file":"cert.pem","key_file":"key.pem"}"#;
        let tls: TlsConfig = serde_json::from_str(json).unwrap();

        assert_eq!(tls.cert_file, "cert.pem");
        assert_eq!(tls.key_file, "key.pem");
        assert!(!tls.http2); // Default is false
    }

    #[test]
    fn test_tls_config_deserialize_with_http2_true() {
        let json = r#"{"cert_file":"cert.pem","key_file":"key.pem","http2":true}"#;
        let tls: TlsConfig = serde_json::from_str(json).unwrap();

        assert!(tls.http2);
    }

    #[test]
    fn test_tls_config_roundtrip() {
        let original = TlsConfig {
            cert_file: "/etc/ssl/certs/server.crt".to_string(),
            key_file: "/etc/ssl/private/server.key".to_string(),
            http2: true,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TlsConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original.cert_file, deserialized.cert_file);
        assert_eq!(original.key_file, deserialized.key_file);
        assert_eq!(original.http2, deserialized.http2);
    }

    #[test]
    fn test_tls_config_clone() {
        let original = TlsConfig {
            cert_file: "a.pem".to_string(),
            key_file: "b.pem".to_string(),
            http2: true,
        };

        let cloned = original.clone();
        assert_eq!(original.cert_file, cloned.cert_file);
        assert_eq!(original.key_file, cloned.key_file);
        assert_eq!(original.http2, cloned.http2);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_server_config_empty_features() {
        let json = r#"{"host":"localhost","port":80,"timeout":10,"features":[]}"#;
        let config: ServerConfig = serde_json::from_str(json).unwrap();
        assert!(config.features.is_empty());
    }

    #[test]
    fn test_server_config_many_features() {
        let features: Vec<String> = (0..100).map(|i| format!("feature_{}", i)).collect();
        let config = ServerConfig {
            features: features.clone(),
            ..Default::default()
        };

        assert_eq!(config.features.len(), 100);
        assert_eq!(config.features[0], "feature_0");
        assert_eq!(config.features[99], "feature_99");
    }

    #[test]
    fn test_server_config_port_boundaries() {
        // Test minimum port
        let config = ServerConfig {
            port: 1,
            ..Default::default()
        };
        assert_eq!(config.port, 1);

        // Test maximum port
        let config = ServerConfig {
            port: 65535,
            ..Default::default()
        };
        assert_eq!(config.port, 65535);
    }

    #[test]
    fn test_server_config_timeout_variations() {
        // Very short timeout
        let config = ServerConfig {
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        assert_eq!(config.timeout, Duration::from_millis(100));

        // Long timeout
        let config = ServerConfig {
            timeout: Duration::from_secs(3600),
            ..Default::default()
        };
        assert_eq!(config.timeout, Duration::from_secs(3600));
    }
}
