//! HTTP client utilities.

use reqwest::header::{HeaderMap, HeaderValue};

use crate::{Result, error::Error};

/// Create a reqwest `ClientBuilder` with default headers and optional proxy.
///
/// Default headers: User-Agent, Accept, Connection, Content-Type
pub fn client_builder(proxy: Option<&str>) -> Result<reqwest::ClientBuilder> {
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("rs_clob_client"));
    headers.insert("Accept", HeaderValue::from_static("*/*"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let mut builder = reqwest::Client::builder().default_headers(headers);

    if let Some(proxy_url) = proxy {
        let proxy = reqwest::Proxy::all(proxy_url)
            .map_err(|err| Error::validation(format!("invalid proxy URL '{proxy_url}': {err}")))?;
        builder = builder.proxy(proxy);
    }

    Ok(builder)
}
