//! SSRF (Server-Side Request Forgery) protection utilities
//!
//! This module provides validation functions to protect against SSRF attacks
//! by checking URLs for private/internal IP addresses and blocked hosts.

use std::net::{IpAddr, Ipv4Addr};
use url::Url;

/// Validate a URL against SSRF attacks
///
/// This function checks that:
/// - The URL is well-formed
/// - The host is not a private/internal IP address
/// - The host is not localhost or a loopback address
/// - The host is not a cloud metadata endpoint
pub fn validate_url_against_ssrf(url_str: &str, context: &str) -> Result<(), String> {
    let url =
        Url::parse(url_str).map_err(|e| format!("{} has invalid URL format: {}", context, e))?;

    // Ensure scheme is http or https
    match url.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(format!(
                "{} must use http:// or https:// scheme, got: {}",
                context, scheme
            ));
        }
    }

    // Get the host
    let host = url
        .host_str()
        .ok_or_else(|| format!("{} URL must have a valid host", context))?;

    // Check for localhost and other local aliases
    let host_lower = host.to_lowercase();
    let blocked_hosts = [
        "localhost",
        "127.0.0.1",
        "::1",
        "[::1]",
        "0.0.0.0",
        "0",
        // AWS metadata endpoint
        "169.254.169.254",
        // Azure metadata endpoint
        "169.254.169.254",
        // GCP metadata endpoint
        "metadata.google.internal",
        "metadata",
        // Common internal hostnames
        "internal",
        "local",
    ];

    for blocked in blocked_hosts {
        if host_lower == blocked || host_lower.ends_with(&format!(".{}", blocked)) {
            return Err(format!(
                "{} URL host '{}' is blocked for security reasons (SSRF protection)",
                context, host
            ));
        }
    }

    // Try to parse as IP address and check for private/internal ranges
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_or_internal_ip(&ip) {
            return Err(format!(
                "{} URL host '{}' is a private/internal IP address (SSRF protection)",
                context, host
            ));
        }
    }

    // Check for IP addresses in brackets (IPv6)
    if host.starts_with('[') && host.ends_with(']') {
        let ip_str = &host[1..host.len() - 1];
        if let Ok(ip) = ip_str.parse::<IpAddr>() {
            if is_private_or_internal_ip(&ip) {
                return Err(format!(
                    "{} URL host '{}' is a private/internal IP address (SSRF protection)",
                    context, host
                ));
            }
        }
    }

    // Check for decimal/octal/hex encoded IP addresses that bypass filters
    // e.g., 2130706433 = 127.0.0.1, 0x7f000001 = 127.0.0.1
    if host.chars().all(|c| c.is_ascii_digit()) {
        // Decimal encoded IP
        if let Ok(num) = host.parse::<u32>() {
            let ip = Ipv4Addr::from(num);
            if is_private_or_internal_ip(&IpAddr::V4(ip)) {
                return Err(format!(
                    "{} URL host '{}' is a decimal-encoded private IP address (SSRF protection)",
                    context, host
                ));
            }
        }
    }

    // Check for hex-encoded IP (0x prefix)
    if host.starts_with("0x") || host.starts_with("0X") {
        if let Ok(num) = u32::from_str_radix(&host[2..], 16) {
            let ip = Ipv4Addr::from(num);
            if is_private_or_internal_ip(&IpAddr::V4(ip)) {
                return Err(format!(
                    "{} URL host '{}' is a hex-encoded private IP address (SSRF protection)",
                    context, host
                ));
            }
        }
    }

    Ok(())
}

/// Check if an IP address is private, internal, or reserved
fn is_private_or_internal_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            // Loopback (127.0.0.0/8)
            ipv4.is_loopback()
            // Private networks (RFC 1918)
            || ipv4.is_private()
            // Link-local (169.254.0.0/16) - includes AWS metadata endpoint
            || ipv4.is_link_local()
            // Broadcast
            || ipv4.is_broadcast()
            // Documentation (TEST-NET)
            || ipv4.is_documentation()
            // Unspecified (0.0.0.0)
            || ipv4.is_unspecified()
            // Shared address space (100.64.0.0/10) - RFC 6598
            || (ipv4.octets()[0] == 100 && (ipv4.octets()[1] & 0xC0) == 64)
            // Reserved (240.0.0.0/4)
            || ipv4.octets()[0] >= 240
        }
        IpAddr::V6(ipv6) => {
            // Loopback (::1)
            ipv6.is_loopback()
            // Unspecified (::)
            || ipv6.is_unspecified()
            // Unique local (fc00::/7)
            || ((ipv6.segments()[0] & 0xfe00) == 0xfc00)
            // Link-local (fe80::/10)
            || ((ipv6.segments()[0] & 0xffc0) == 0xfe80)
            // IPv4-mapped addresses - check the embedded IPv4
            || ipv6.to_ipv4_mapped().is_some_and(|ipv4| {
                ipv4.is_loopback() || ipv4.is_private() || ipv4.is_link_local()
            })
        }
    }
}
