use reqwest::{
    Client as ReqwestClient, Method, Request, StatusCode,
    header::{HeaderMap, HeaderValue},
};
use serde::de::DeserializeOwned;
use url::Url;

use super::types::{
    ListTeamsRequest, ListTeamsResponse, RelatedTagsByIdRequest, RelatedTagsBySlugRequest,
    SportsMarketTypesResponse, SportsMetadataResponse, Tag, TagRelationship, TagsRequest,
};
use crate::Result;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Client {
    host: Url,
    client: ReqwestClient,
}

impl Default for Client {
    fn default() -> Self {
        Client::new("https://gamma-api.polymarket.com")
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

    async fn request<Response: DeserializeOwned>(
        &self,
        mut request: Request,
        headers: Option<HeaderMap>,
    ) -> Result<Response> {
        let method = request.method().clone();
        let path = request.url().path().to_owned();

        if let Some(h) = headers {
            *request.headers_mut() = h;
        }

        let response = self.client.execute(request).await?;
        let status_code = response.status();

        if !status_code.is_success() {
            let message = response.text().await.unwrap_or_default();

            return Err(Error::status(status_code, method, path, message));
        }

        match response.json::<Option<Response>>().await? {
            Some(response) => Ok(response),
            None => Err(Error::status(
                StatusCode::NOT_FOUND,
                method,
                path,
                "Unable to find requested resource",
            )),
        }
    }

    #[must_use]
    pub fn host(&self) -> &Url {
        &self.host
    }

    #[must_use]
    fn client(&self) -> &ReqwestClient {
        &self.client
    }

    pub async fn teams(&self, request: &ListTeamsRequest) -> Result<ListTeamsResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}teams", self.host()))
            .query(&request)
            .build()?;

        self.request(request, None).await
    }

    pub async fn sports(&self) -> Result<SportsMetadataResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}sports", self.host()))
            .build()?;

        self.request(request, None).await
    }

    pub async fn sports_market_types(&self) -> Result<SportsMarketTypesResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}sports/market-types", self.host()))
            .build()?;

        self.request(request, None).await
    }

    pub async fn tags(&self, request: &TagsRequest) -> Result<Vec<Tag>> {
        let request = self
            .client()
            .request(Method::GET, format!("{}tags", self.host()))
            .query(request)
            .build()?;

        self.request(request, None).await
    }

    pub async fn tag_by_id(&self, id: u32, include_template: Option<bool>) -> Result<Tag> {
        let mut request = self
            .client()
            .request(Method::GET, format!("{}tags/{}", self.host(), id));

        if let Some(include) = include_template {
            request = request.query(&[("include_template", include)]);
        }

        self.request(request.build()?, None).await
    }

    pub async fn tag_by_slug(&self, slug: &str, include_template: Option<bool>) -> Result<Tag> {
        let mut request = self
            .client()
            .request(Method::GET, format!("{}tags/slug/{}", self.host(), slug));

        if let Some(include) = include_template {
            request = request.query(&[("include_template", include)]);
        }

        self.request(request.build()?, None).await
    }

    pub async fn tag_relationships_by_id(
        &self,
        request: &RelatedTagsByIdRequest,
    ) -> Result<Vec<TagRelationship>> {
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}tags/{}/related-tags", self.host(), request.id),
            )
            .query(request)
            .build()?;

        self.request(request, None).await
    }

    pub async fn tag_relationships_by_slug(
        &self,
        request: &RelatedTagsBySlugRequest,
    ) -> Result<Vec<TagRelationship>> {
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}tags/slug/{}/related-tags", self.host(), request.slug),
            )
            .query(request)
            .build()?;

        self.request(request, None).await
    }

    pub async fn related_tags_by_id(&self, request: &RelatedTagsByIdRequest) -> Result<Vec<Tag>> {
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}tags/{}/related-tags/tags", self.host(), request.id),
            )
            .query(request)
            .build()?;

        self.request(request, None).await
    }

    pub async fn related_tags_by_slug(
        &self,
        request: &RelatedTagsBySlugRequest,
    ) -> Result<Vec<Tag>> {
        let request = self
            .client()
            .request(
                Method::GET,
                format!(
                    "{}tags/slug/{}/related-tags/tags",
                    self.host(),
                    request.slug
                ),
            )
            .query(request)
            .build()?;

        self.request(request, None).await
    }
}
