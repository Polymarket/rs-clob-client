//! Comprehensive Gamma API endpoint explorer.
//!
//! This example dynamically tests all Gamma API endpoints by:
//! 1. Fetching lists first (events, markets, tags, etc.)
//! 2. Extracting real IDs/slugs from responses
//! 3. Using those IDs for subsequent lookups
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example gamma --features gamma,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=gamma.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example gamma --features gamma,tracing
//! ```

use std::fs::File;

use polymarket_client_sdk::gamma::Client;
use polymarket_client_sdk::gamma::types::ParentEntityType;
use polymarket_client_sdk::gamma::types::request::{
    CommentsByIdRequest, CommentsByUserAddressRequest, CommentsRequest, EventByIdRequest,
    EventBySlugRequest, EventTagsRequest, EventsRequest, MarketByIdRequest, MarketBySlugRequest,
    MarketTagsRequest, MarketsRequest, PublicProfileRequest, RelatedTagsByIdRequest,
    RelatedTagsBySlugRequest, SearchRequest, SeriesByIdRequest, SeriesListRequest, TagByIdRequest,
    TagBySlugRequest, TagsRequest, TeamsRequest,
};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Ok(path) = std::env::var("LOG_FILE") {
        let file = File::create(path)?;
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file)
                    .with_ansi(false),
            )
            .init();
    } else {
        tracing_subscriber::fmt::init();
    }

    let client = Client::default();

    match client.status().await {
        Ok(s) => info!(endpoint = "status", result = %s),
        Err(e) => debug!(endpoint = "status", error = %e),
    }

    match client.sports().await {
        Ok(v) => info!(endpoint = "sports", count = v.len()),
        Err(e) => debug!(endpoint = "sports", error = %e),
    }

    match client.sports_market_types().await {
        Ok(v) => info!(
            endpoint = "sports_market_types",
            count = v.market_types.len()
        ),
        Err(e) => debug!(endpoint = "sports_market_types", error = %e),
    }

    match client
        .teams(&TeamsRequest::builder().limit(5).build())
        .await
    {
        Ok(v) => info!(endpoint = "teams", count = v.len()),
        Err(e) => debug!(endpoint = "teams", error = %e),
    }

    let tags_result = client.tags(&TagsRequest::builder().limit(10).build()).await;
    match &tags_result {
        Ok(v) => info!(endpoint = "tags", count = v.len()),
        Err(e) => debug!(endpoint = "tags", error = %e),
    }

    // Use "politics" tag - known to have related tags
    let tag_slug = "politics";
    let tag_result = client
        .tag_by_slug(&TagBySlugRequest::builder().slug(tag_slug).build())
        .await;
    let tag_id = match &tag_result {
        Ok(tag) => {
            info!(endpoint = "tag_by_slug", slug = tag_slug, id = %tag.id);
            Some(tag.id.clone())
        }
        Err(e) => {
            debug!(endpoint = "tag_by_slug", slug = tag_slug, error = %e);
            None
        }
    };

    if let Some(id) = &tag_id {
        match client
            .tag_by_id(&TagByIdRequest::builder().id(id).build())
            .await
        {
            Ok(_) => info!(endpoint = "tag_by_id", id = %id),
            Err(e) => debug!(endpoint = "tag_by_id", id = %id, error = %e),
        }

        match client
            .related_tags_by_id(&RelatedTagsByIdRequest::builder().id(id).build())
            .await
        {
            Ok(v) => info!(endpoint = "related_tags_by_id", id = %id, count = v.len()),
            Err(e) => debug!(endpoint = "related_tags_by_id", id = %id, error = %e),
        }

        match client
            .tags_related_to_tag_by_id(&RelatedTagsByIdRequest::builder().id(id).build())
            .await
        {
            Ok(v) => info!(endpoint = "tags_related_to_tag_by_id", id = %id, count = v.len()),
            Err(e) => debug!(endpoint = "tags_related_to_tag_by_id", id = %id, error = %e),
        }
    }

    match client
        .related_tags_by_slug(&RelatedTagsBySlugRequest::builder().slug(tag_slug).build())
        .await
    {
        Ok(v) => info!(
            endpoint = "related_tags_by_slug",
            slug = tag_slug,
            count = v.len()
        ),
        Err(e) => debug!(endpoint = "related_tags_by_slug", slug = tag_slug, error = %e),
    }

    match client
        .tags_related_to_tag_by_slug(&RelatedTagsBySlugRequest::builder().slug(tag_slug).build())
        .await
    {
        Ok(v) => info!(
            endpoint = "tags_related_to_tag_by_slug",
            slug = tag_slug,
            count = v.len()
        ),
        Err(e) => debug!(endpoint = "tags_related_to_tag_by_slug", slug = tag_slug, error = %e),
    }

    let events_result = client
        .events(
            &EventsRequest::builder()
                .active(true)
                .limit(20)
                .order(vec!["volume".to_owned()])
                .ascending(false)
                .build(),
        )
        .await;

    // Find an event with comments
    let (event_with_comments, any_event) = match &events_result {
        Ok(events) => {
            info!(endpoint = "events", count = events.len());
            let with_comments = events
                .iter()
                .find(|e| e.comment_count.unwrap_or(0) > 0)
                .map(|e| (e.id.clone(), e.slug.clone(), e.comment_count.unwrap_or(0)));
            let any = events.first().map(|e| (e.id.clone(), e.slug.clone()));
            (with_comments, any)
        }
        Err(e) => {
            debug!(endpoint = "events", error = %e);
            (None, None)
        }
    };

    if let Some((event_id, event_slug)) = &any_event {
        match client
            .event_by_id(&EventByIdRequest::builder().id(event_id).build())
            .await
        {
            Ok(_) => info!(endpoint = "event_by_id", id = %event_id),
            Err(e) => debug!(endpoint = "event_by_id", id = %event_id, error = %e),
        }

        match client
            .event_tags(&EventTagsRequest::builder().id(event_id).build())
            .await
        {
            Ok(v) => info!(endpoint = "event_tags", id = %event_id, count = v.len()),
            Err(e) => debug!(endpoint = "event_tags", id = %event_id, error = %e),
        }

        if let Some(slug) = event_slug {
            match client
                .event_by_slug(&EventBySlugRequest::builder().slug(slug).build())
                .await
            {
                Ok(_) => info!(endpoint = "event_by_slug", slug = %slug),
                Err(e) => debug!(endpoint = "event_by_slug", slug = %slug, error = %e),
            }
        }
    }

    let markets_result = client
        .markets(&MarketsRequest::builder().closed(false).limit(10).build())
        .await;

    let (market_id, market_slug) = match &markets_result {
        Ok(markets) => {
            info!(endpoint = "markets", count = markets.len());
            markets
                .first()
                .map_or((None, None), |m| (Some(m.id.clone()), m.slug.clone()))
        }
        Err(e) => {
            debug!(endpoint = "markets", error = %e);
            (None, None)
        }
    };

    // Test multiple slugs - verifies repeated query params work (issue #147)
    if let Ok(markets) = &markets_result {
        let slugs: Vec<String> = markets
            .iter()
            .filter_map(|m| m.slug.clone())
            .take(3)
            .collect();

        if slugs.len() >= 2 {
            match client
                .markets(&MarketsRequest::builder().slug(slugs.clone()).build())
                .await
            {
                Ok(v) => info!(
                    endpoint = "markets_multiple_slugs",
                    slugs = ?slugs,
                    count = v.len(),
                    "verified repeated query params work"
                ),
                Err(e) => debug!(endpoint = "markets_multiple_slugs", slugs = ?slugs, error = %e),
            }
        }
    }

    if let Some(id) = &market_id {
        match client
            .market_by_id(&MarketByIdRequest::builder().id(id).build())
            .await
        {
            Ok(_) => info!(endpoint = "market_by_id", id = %id),
            Err(e) => debug!(endpoint = "market_by_id", id = %id, error = %e),
        }

        match client
            .market_tags(&MarketTagsRequest::builder().id(id).build())
            .await
        {
            Ok(v) => info!(endpoint = "market_tags", id = %id, count = v.len()),
            Err(e) => debug!(endpoint = "market_tags", id = %id, error = %e),
        }
    }

    if let Some(slug) = &market_slug {
        match client
            .market_by_slug(&MarketBySlugRequest::builder().slug(slug).build())
            .await
        {
            Ok(_) => info!(endpoint = "market_by_slug", slug = %slug),
            Err(e) => debug!(endpoint = "market_by_slug", slug = %slug, error = %e),
        }
    }

    let series_result = client
        .series(
            &SeriesListRequest::builder()
                .limit(10)
                .order("volume".to_owned())
                .ascending(false)
                .build(),
        )
        .await;

    let series_id = match &series_result {
        Ok(series) => {
            info!(endpoint = "series", count = series.len());
            series.first().map(|s| s.id.clone())
        }
        Err(e) => {
            debug!(endpoint = "series", error = %e);
            None
        }
    };

    if let Some(id) = &series_id {
        match client
            .series_by_id(&SeriesByIdRequest::builder().id(id).build())
            .await
        {
            Ok(_) => info!(endpoint = "series_by_id", id = %id),
            Err(e) => debug!(endpoint = "series_by_id", id = %id, error = %e),
        }
    }

    let (comment_id, user_address) = if let Some((event_id, _, comment_count)) =
        &event_with_comments
    {
        let comments_result = client
            .comments(
                &CommentsRequest::builder()
                    .parent_entity_type(ParentEntityType::Event)
                    .parent_entity_id(event_id)
                    .limit(10)
                    .build(),
            )
            .await;

        match &comments_result {
            Ok(comments) => {
                info!(endpoint = "comments", event_id = %event_id, expected = comment_count, count = comments.len());
                comments
                    .first()
                    .map_or((None, None), |c| (Some(c.id.clone()), c.user_address))
            }
            Err(e) => {
                debug!(endpoint = "comments", event_id = %event_id, error = %e);
                (None, None)
            }
        }
    } else {
        debug!(
            endpoint = "comments",
            "skipped - no event with comments found"
        );
        (None, None)
    };

    if let Some(id) = &comment_id {
        match client
            .comments_by_id(&CommentsByIdRequest::builder().id(id).build())
            .await
        {
            Ok(v) => info!(endpoint = "comments_by_id", id = %id, count = v.len()),
            Err(e) => debug!(endpoint = "comments_by_id", id = %id, error = %e),
        }
    }

    if let Some(addr) = user_address {
        match client
            .comments_by_user_address(
                &CommentsByUserAddressRequest::builder()
                    .user_address(addr)
                    .limit(5)
                    .build(),
            )
            .await
        {
            Ok(v) => info!(endpoint = "comments_by_user_address", address = %addr, count = v.len()),
            Err(e) => debug!(endpoint = "comments_by_user_address", address = %addr, error = %e),
        }
    }

    // Use the user_address from comments if available
    if let Some(profile_address) = user_address {
        match client
            .public_profile(
                &PublicProfileRequest::builder()
                    .address(profile_address)
                    .build(),
            )
            .await
        {
            Ok(p) => {
                let name = p.pseudonym.as_deref().unwrap_or("anonymous");
                info!(endpoint = "public_profile", address = %profile_address, name = %name);
            }
            Err(e) => debug!(endpoint = "public_profile", address = %profile_address, error = %e),
        }
    }

    let query = "trump";
    match client
        .search(&SearchRequest::builder().q(query).build())
        .await
    {
        Ok(r) => {
            let events = r.events.map_or(0, |e| e.len());
            let tags = r.tags.map_or(0, |t| t.len());
            let profiles = r.profiles.map_or(0, |p| p.len());
            info!(
                endpoint = "search",
                query = query,
                events = events,
                tags = tags,
                profiles = profiles
            );
        }
        Err(e) => debug!(endpoint = "search", query = query, error = %e),
    }

    Ok(())
}
