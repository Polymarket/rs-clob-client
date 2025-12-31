//! Proxy connection utilities for WebSocket clients.

use std::fmt::Write as _;

use base64::Engine as _;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpStream;
use url::Url;

use crate::{Result, error::Error};

/// Read proxy URL from environment variables for a given target host.
///
/// Returns `None` if no proxy env vars are set or target matches `NO_PROXY`.
///
/// Priority: `all_proxy` > `ALL_PROXY` > `https_proxy` > `HTTPS_PROXY` > `http_proxy` > `HTTP_PROXY`
#[must_use]
pub fn from_env(target_host: &str) -> Option<String> {
    if is_no_proxy(target_host) {
        return None;
    }
    std::env::var("all_proxy")
        .or_else(|_| std::env::var("ALL_PROXY"))
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("HTTPS_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .ok()
}

/// Check if host matches `NO_PROXY` patterns.
///
/// Patterns: comma-separated list of hosts/domains (e.g., `"localhost,.example.com,192.168.1.1"`)
fn is_no_proxy(host: &str) -> bool {
    let no_proxy = std::env::var("no_proxy")
        .or_else(|_| std::env::var("NO_PROXY"))
        .unwrap_or_default();

    if no_proxy == "*" {
        return true;
    }

    for pattern in no_proxy.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(suffix) = pattern.strip_prefix('.') {
            // Domain suffix match: .example.com matches foo.example.com
            if host.ends_with(pattern) || host == suffix {
                return true;
            }
        } else if host == pattern || host.ends_with(&format!(".{pattern}")) {
            return true;
        }
    }
    false
}

/// Connect to a WebSocket target through a proxy.
///
/// Returns a raw `TcpStream` connected to the target via the proxy.
/// The caller is responsible for upgrading this to a WebSocket connection.
///
/// Supports:
/// - SOCKS5 proxies: `socks5://[user:pass@]host:port`
/// - HTTP CONNECT proxies: `http://[user:pass@]host:port`
pub async fn connect(endpoint: &str, proxy_url: &str) -> Result<TcpStream> {
    let proxy = Url::parse(proxy_url)
        .map_err(|err| Error::validation(format!("invalid proxy URL '{proxy_url}': {err}")))?;

    let target = Url::parse(endpoint)
        .map_err(|err| Error::validation(format!("invalid WebSocket URL '{endpoint}': {err}")))?;

    let target_host = target
        .host_str()
        .ok_or_else(|| Error::validation("WebSocket URL missing host"))?;
    let target_port = target.port_or_known_default().unwrap_or(443);

    match proxy.scheme() {
        "socks5" | "socks5h" => connect_socks5(&proxy, target_host, target_port).await,
        "http" | "https" => connect_http_tunnel(&proxy, target_host, target_port).await,
        scheme => Err(Error::validation(format!(
            "unsupported proxy scheme '{scheme}', expected socks5 or http"
        ))),
    }
}

/// Connect through a SOCKS5 proxy.
async fn connect_socks5(proxy: &Url, target_host: &str, target_port: u16) -> Result<TcpStream> {
    let proxy_host = proxy
        .host_str()
        .ok_or_else(|| Error::validation("SOCKS5 proxy URL missing host"))?;
    let proxy_port = proxy.port().unwrap_or(1080);
    let proxy_addr = format!("{proxy_host}:{proxy_port}");

    let stream = if proxy.username().is_empty() {
        tokio_socks::tcp::Socks5Stream::connect(proxy_addr.as_str(), (target_host, target_port))
            .await
            .map_err(|err| Error::validation(format!("SOCKS5 connection failed: {err}")))?
    } else {
        let password = proxy.password().unwrap_or("");
        tokio_socks::tcp::Socks5Stream::connect_with_password(
            proxy_addr.as_str(),
            (target_host, target_port),
            proxy.username(),
            password,
        )
        .await
        .map_err(|err| Error::validation(format!("SOCKS5 connection failed: {err}")))?
    };

    Ok(stream.into_inner())
}

/// Connect through an HTTP CONNECT tunnel.
async fn connect_http_tunnel(
    proxy: &Url,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream> {
    let proxy_host = proxy
        .host_str()
        .ok_or_else(|| Error::validation("HTTP proxy URL missing host"))?;
    let proxy_port = proxy.port().unwrap_or(8080);

    // Connect to the proxy
    let mut stream = TcpStream::connect((proxy_host, proxy_port))
        .await
        .map_err(|err| Error::validation(format!("failed to connect to HTTP proxy: {err}")))?;

    // Build CONNECT request
    let mut connect_request = format!(
        "CONNECT {target_host}:{target_port} HTTP/1.1\r\n\
         Host: {target_host}:{target_port}\r\n"
    );

    // Add proxy authentication if provided
    if !proxy.username().is_empty() {
        let credentials = format!("{}:{}", proxy.username(), proxy.password().unwrap_or(""));
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
        write!(connect_request, "Proxy-Authorization: Basic {encoded}\r\n")
            .map_err(|err| Error::validation(format!("failed to format CONNECT request: {err}")))?;
    }

    connect_request.push_str("\r\n");

    // Send CONNECT request
    stream
        .write_all(connect_request.as_bytes())
        .await
        .map_err(|err| Error::validation(format!("failed to send CONNECT request: {err}")))?;

    // Read response
    let mut buf = [0_u8; 1024];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|err| Error::validation(format!("failed to read CONNECT response: {err}")))?;

    let response = String::from_utf8_lossy(&buf[..n]);

    // Check for successful tunnel establishment (HTTP 200)
    if !response.starts_with("HTTP/1.1 200") && !response.starts_with("HTTP/1.0 200") {
        return Err(Error::validation(format!(
            "HTTP CONNECT tunnel failed: {}",
            response.lines().next().unwrap_or("unknown error")
        )));
    }

    Ok(stream)
}
