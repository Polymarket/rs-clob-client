use reqwest::{
    Client as ReqwestClient, Method, Request, StatusCode,
    header::{HeaderMap, HeaderValue},
};
use serde::de::DeserializeOwned;
use url::Url;

use super::types::{
    Activity, ActivityRequest, BuilderLeaderboardEntry, BuilderLeaderboardRequest,
    BuilderVolumeEntry, BuilderVolumeRequest, ClosedPosition, ClosedPositionsRequest,
    HealthResponse, HoldersRequest, LiveVolume, LiveVolumeRequest, MetaHolder, OpenInterest,
    OpenInterestRequest, Position, PositionsRequest, QueryParams, Trade, Traded, TradedRequest,
    TraderLeaderboardEntry, TraderLeaderboardRequest, TradesRequest, Value, ValueRequest,
};
use crate::Result;
use crate::error::Error;

fn to_query_string(params: &[(&'static str, String)]) -> String {
    if params.is_empty() {
        String::new()
    } else {
        let encoded: String = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params.iter().map(|(k, v)| (*k, v.as_str())))
            .finish();
        format!("?{encoded}")
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    host: Url,
    client: ReqwestClient,
}

impl Default for Client {
    fn default() -> Self {
        Client::new("https://data-api.polymarket.com")
            .expect("Client with default endpoint should succeed")
    }
}

impl Client {
    pub fn new(host: &str) -> Result<Client> {
        let mut headers = HeaderMap::new();

        headers.insert("User-Agent", HeaderValue::from_static("rs_clob_client"));
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        let client = ReqwestClient::builder().default_headers(headers).build()?;

        Ok(Self {
            host: Url::parse(host)?,
            client,
        })
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip(self, request, headers),
            fields(method, path, status_code)
        )
    )]
    async fn request<Response: DeserializeOwned>(
        &self,
        mut request: Request,
        headers: Option<HeaderMap>,
    ) -> Result<Response> {
        let method = request.method().clone();
        let path = request.url().path().to_owned();

        #[cfg(feature = "tracing")]
        {
            let span = tracing::Span::current();
            span.record("method", method.as_str());
            span.record("path", path.as_str());
        }

        if let Some(h) = headers {
            *request.headers_mut() = h;
        }

        let response = self.client.execute(request).await?;
        let status_code = response.status();

        #[cfg(feature = "tracing")]
        tracing::Span::current().record("status_code", status_code.as_u16());

        if !status_code.is_success() {
            let message = response.text().await.unwrap_or_default();

            #[cfg(feature = "tracing")]
            tracing::warn!(
                status = %status_code,
                method = %method,
                path = %path,
                message = %message,
                "Data API request failed"
            );

            return Err(Error::status(status_code, method, path, message));
        }

        if let Some(response) = response.json::<Option<Response>>().await? {
            Ok(response)
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!(method = %method, path = %path, "Data API resource not found");
            Err(Error::status(
                StatusCode::NOT_FOUND,
                method,
                path,
                "Unable to find requested resource",
            ))
        }
    }

    #[must_use]
    pub fn host(&self) -> &Url {
        &self.host
    }

    async fn get<Req: QueryParams, Res: DeserializeOwned>(
        &self,
        path: &str,
        req: &Req,
    ) -> Result<Res> {
        let params = to_query_string(&req.query_params());
        let request = self
            .client
            .request(Method::GET, format!("{}{path}{params}", self.host))
            .build()?;
        self.request(request, None).await
    }

    pub async fn health(&self) -> Result<HealthResponse> {
        self.get("", &()).await
    }

    pub async fn positions(&self, req: &PositionsRequest) -> Result<Vec<Position>> {
        self.get("positions", req).await
    }

    pub async fn trades(&self, req: &TradesRequest) -> Result<Vec<Trade>> {
        self.get("trades", req).await
    }

    pub async fn activity(&self, req: &ActivityRequest) -> Result<Vec<Activity>> {
        self.get("activity", req).await
    }

    pub async fn holders(&self, req: &HoldersRequest) -> Result<Vec<MetaHolder>> {
        self.get("holders", req).await
    }

    pub async fn value(&self, req: &ValueRequest) -> Result<Vec<Value>> {
        self.get("value", req).await
    }

    pub async fn closed_positions(
        &self,
        req: &ClosedPositionsRequest,
    ) -> Result<Vec<ClosedPosition>> {
        self.get("closed-positions", req).await
    }

    pub async fn leaderboard(
        &self,
        req: &TraderLeaderboardRequest,
    ) -> Result<Vec<TraderLeaderboardEntry>> {
        self.get("v1/leaderboard", req).await
    }

    pub async fn traded(&self, req: &TradedRequest) -> Result<Traded> {
        self.get("traded", req).await
    }

    pub async fn open_interest(&self, req: &OpenInterestRequest) -> Result<Vec<OpenInterest>> {
        self.get("oi", req).await
    }

    pub async fn live_volume(&self, req: &LiveVolumeRequest) -> Result<Vec<LiveVolume>> {
        self.get("live-volume", req).await
    }

    pub async fn builder_leaderboard(
        &self,
        req: &BuilderLeaderboardRequest,
    ) -> Result<Vec<BuilderLeaderboardEntry>> {
        self.get("v1/builders/leaderboard", req).await
    }

    pub async fn builder_volume(
        &self,
        req: &BuilderVolumeRequest,
    ) -> Result<Vec<BuilderVolumeEntry>> {
        self.get("v1/builders/volume", req).await
    }
}
