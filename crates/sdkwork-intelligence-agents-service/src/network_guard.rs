//! Shared outbound-URL guard against server-side request forgery.
//!
//! Every server feature that fetches a tenant-supplied URL — the generated
//! media fetch path and the webhook delivery path — must never connect to
//! internal network space: loopback, private ranges, link-local addresses
//! (cloud metadata endpoints), or multicast. This module owns that policy
//! once so the delivery paths cannot drift apart.

use std::net::IpAddr;

/// Error returned when an outbound URL must not be dialed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundUrlError(pub String);

impl std::fmt::Display for OutboundUrlError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for OutboundUrlError {}

/// Parses an absolute `http`/`https` URL and verifies its host is reachable
/// public network space before the caller dials it.
///
/// IP literals are checked directly; hostnames are rejected for known
/// internal suffixes and, when resolvable, for internal resolved addresses.
/// The DNS rebinding window between resolution and connection is bounded by
/// the caller's fetch/delivery timeout.
pub(crate) async fn ensure_outbound_target_is_public(
    raw: &str,
) -> Result<reqwest::Url, OutboundUrlError> {
    let url = reqwest::Url::parse(raw)
        .map_err(|error| OutboundUrlError(format!("invalid outbound URL: {error}")))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(OutboundUrlError("outbound URL must use http or https".to_string()));
    }
    let host = url
        .host_str()
        .ok_or_else(|| OutboundUrlError("outbound URL has no host".to_string()))?;
    if host_is_unreachable_from_server(host).await {
        return Err(OutboundUrlError(format!(
            "outbound URL host {host} is not reachable from the server"
        )));
    }
    Ok(url)
}

/// Rejects hosts that resolve to internal or link-local network space.
///
/// Resolution failure is treated as reachable-unknown; the caller's timeout
/// still bounds the attempt.
async fn host_is_unreachable_from_server(host: &str) -> bool {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_internal_ip(ip);
    }
    let host = host.to_ascii_lowercase();
    if host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.ends_with(".localdomain")
    {
        return true;
    }
    if let Ok(addresses) = tokio::net::lookup_host((host.as_str(), 0)).await {
        return addresses.map(|address| address.ip()).any(is_internal_ip);
    }
    false
}

/// Whether an address belongs to network space the server must never dial:
/// loopback, private, link-local, unspecified, multicast or broadcast.
pub(crate) fn is_internal_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_multicast()
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || v6.is_unique_local()
                || v6.is_unicast_link_local()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_internal_network_targets() {
        for url in [
            "http://127.0.0.1:8080/secret",
            "http://10.0.0.5/metadata",
            "http://192.168.1.1/internal",
            "http://169.254.169.254/latest/meta-data",
            "http://[::1]/loopback",
            "http://localhost:9000/admin",
            "ftp://cdn.example.com/file.png",
            "file:///etc/passwd",
            "http://metadata.internal/",
            "http://169.254.169.254",
        ] {
            let error = ensure_outbound_target_is_public(url)
                .await
                .expect_err("internal outbound URL must be rejected");
            assert!(
                !error.0.is_empty(),
                "{url} must fail with an actionable error"
            );
        }
    }

    #[tokio::test]
    async fn accepts_public_https_targets_for_validation() {
        // The validation stage accepts public URLs; the actual network fetch
        // is never attempted here, so any send failure is the caller's
        // concern rather than a validation rejection.
        let url = "https://cdn.example.com/generated/image.png";
        match ensure_outbound_target_is_public(url).await {
            Ok(parsed) => assert_eq!(parsed.as_str(), url),
            Err(error) => panic!("public URL must pass validation, got: {error}"),
        }
    }

    #[test]
    fn internal_ip_detection_covers_ipv4_and_ipv6() {
        assert!(is_internal_ip("127.0.0.1".parse().expect("ip")));
        assert!(is_internal_ip("10.1.2.3".parse().expect("ip")));
        assert!(is_internal_ip("172.16.0.1".parse().expect("ip")));
        assert!(is_internal_ip("192.168.1.1".parse().expect("ip")));
        assert!(is_internal_ip("169.254.1.1".parse().expect("ip")));
        assert!(is_internal_ip("::1".parse().expect("ip")));
        assert!(is_internal_ip("fd00::1".parse().expect("ip")));
        assert!(is_internal_ip("fe80::1".parse().expect("ip")));
        assert!(!is_internal_ip("8.8.8.8".parse().expect("ip")));
        assert!(!is_internal_ip("93.184.216.34".parse().expect("ip")));
        assert!(!is_internal_ip("2606:2800:220:1::1".parse().expect("ip")));
    }
}
