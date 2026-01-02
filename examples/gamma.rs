//! Live demo of the Polymarket Gamma API client.
//!
//! This example calls all Gamma API endpoints and prints the results. Run with:
//!
//! ```sh
//! RUST_LOG=info cargo run --example gamma --features gamma,tracing
//! ```
//!
//! The demo dynamically discovers valid IDs from list endpoints to ensure
//! detail endpoints are called with real data.

use polymarket_client_sdk::gamma::Client;
use polymarket_client_sdk::gamma::types::ParentEntityType;
use polymarket_client_sdk::gamma::types::request::{
    CommentsByUserAddressRequest, CommentsRequest, EventByIdRequest, EventBySlugRequest,
    EventTagsRequest, EventsRequest, MarketByIdRequest, MarketBySlugRequest, MarketTagsRequest,
    MarketsRequest, PublicProfileRequest, RelatedTagsByIdRequest, RelatedTagsBySlugRequest,
    SearchRequest, SeriesByIdRequest, SeriesListRequest, TagByIdRequest, TagBySlugRequest,
    TagsRequest, TeamsRequest,
};
use polymarket_client_sdk::gamma::types::response::{
    Comment, Event, Market, RelatedTag, Series, Tag,
};
use tracing::{error, info, warn};

fn first_event_id(events: &[Event]) -> Option<&str> {
    events.first().map(|e| e.id.as_str())
}

fn first_event_slug(events: &[Event]) -> Option<&str> {
    let e = events.first()?;
    e.slug.as_deref()
}

fn first_market_id(markets: &[Market]) -> Option<&str> {
    markets.first().map(|m| m.id.as_str())
}

fn first_market_slug(markets: &[Market]) -> Option<&str> {
    let m = markets.first()?;
    m.slug.as_deref()
}

fn first_series_id(series_list: &[Series]) -> Option<&str> {
    series_list.first().map(|s| s.id.as_str())
}

/// Find a tag with related tags (try first few tags).
async fn find_tag_with_related(client: &Client, tags: &[Tag]) -> Option<(Tag, Vec<RelatedTag>)> {
    for tag in tags.iter().take(10) {
        let request = RelatedTagsByIdRequest::builder().id(&tag.id).build();
        if let Ok(related) = client.related_tags_by_id(&request).await
            && !related.is_empty()
        {
            return Some((tag.clone(), related));
        }
    }
    None
}

/// Find an event with comments (try first few events).
async fn find_event_with_comments(
    client: &Client,
    events: &[Event],
) -> Option<(Event, Vec<Comment>)> {
    for event in events.iter().take(10) {
        let request = CommentsRequest::builder()
            .limit(10)
            .parent_entity_type(ParentEntityType::Event)
            .parent_entity_id(&event.id)
            .build();
        if let Ok(comments) = client.comments(&request).await
            && !comments.is_empty()
        {
            return Some((event.clone(), comments));
        }
    }
    None
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();

    info!(host = %client.host(), "Starting Gamma API demo");

    info!("═══ Health Check ═══");
    match client.status().await {
        Ok(status) => info!(endpoint = "status", result = %status),
        Err(e) => error!(endpoint = "status", error = %e),
    }

    info!("═══ Tags ═══");
    let tags_request = TagsRequest::builder().limit(20).build();
    let tags = client.tags(&tags_request).await;
    match &tags {
        Ok(t) => info!(endpoint = "tags", count = t.len()),
        Err(e) => error!(endpoint = "tags", error = %e),
    }

    // Find a tag with related tags for better demo output
    let tag_with_related = match &tags {
        Ok(t) => find_tag_with_related(&client, t).await,
        Err(_) => None,
    };

    if let Some((tag, related)) = tag_with_related {
        let label = tag.label.as_deref().unwrap_or("(no label)");
        info!(endpoint = "tag_by_id", label = %label);
        info!(endpoint = "related_tags_by_id", count = related.len());

        if let Some(slug) = &tag.slug {
            let tag_by_slug_req = TagBySlugRequest::builder().slug(slug).build();
            match client.tag_by_slug(&tag_by_slug_req).await {
                Ok(t) => info!(
                    endpoint = "tag_by_slug",
                    label = t.label.as_deref().unwrap_or("(no label)")
                ),
                Err(e) => error!(endpoint = "tag_by_slug", error = %e),
            }

            let related_by_slug_req = RelatedTagsBySlugRequest::builder().slug(slug).build();
            match client.related_tags_by_slug(&related_by_slug_req).await {
                Ok(r) => info!(endpoint = "related_tags_by_slug", count = r.len()),
                Err(e) => error!(endpoint = "related_tags_by_slug", error = %e),
            }

            match client
                .tags_related_to_tag_by_slug(&related_by_slug_req)
                .await
            {
                Ok(r) => info!(endpoint = "tags_related_to_tag_by_slug", count = r.len()),
                Err(e) => error!(endpoint = "tags_related_to_tag_by_slug", error = %e),
            }
        } else {
            warn!(endpoint = "tag_by_slug", reason = "no tag slug available");
            warn!(
                endpoint = "related_tags_by_slug",
                reason = "no tag slug available"
            );
            warn!(
                endpoint = "tags_related_to_tag_by_slug",
                reason = "no tag slug available"
            );
        }

        let related_tag_objs_req = RelatedTagsByIdRequest::builder().id(&tag.id).build();
        match client
            .tags_related_to_tag_by_id(&related_tag_objs_req)
            .await
        {
            Ok(r) => info!(endpoint = "tags_related_to_tag_by_id", count = r.len()),
            Err(e) => error!(endpoint = "tags_related_to_tag_by_id", error = %e),
        }
    } else {
        // Fall back to first tag if none have related tags
        if let Ok(t) = &tags {
            if let Some(first_tag) = t.first() {
                let request = TagByIdRequest::builder().id(&first_tag.id).build();
                match client.tag_by_id(&request).await {
                    Ok(t) => info!(
                        endpoint = "tag_by_id",
                        label = t.label.as_deref().unwrap_or("(no label)")
                    ),
                    Err(e) => error!(endpoint = "tag_by_id", error = %e),
                }
                info!(
                    endpoint = "related_tags_by_id",
                    count = 0,
                    note = "no tags with relations found"
                );
            } else {
                warn!(endpoint = "tag_by_id", reason = "no tags available");
            }
        }
        warn!(
            endpoint = "tag_by_slug",
            reason = "no tag with related tags found"
        );
        warn!(
            endpoint = "related_tags_by_slug",
            reason = "no tag with related tags found"
        );
        warn!(
            endpoint = "tags_related_to_tag_by_slug",
            reason = "no tag with related tags found"
        );
        warn!(
            endpoint = "tags_related_to_tag_by_id",
            reason = "no tag with related tags found"
        );
    }

    info!("═══ Events ═══");
    let events_request = EventsRequest::builder().active(true).limit(10).build();
    let events = client.events(&events_request).await;
    match &events {
        Ok(ev) => info!(endpoint = "events", count = ev.len()),
        Err(e) => error!(endpoint = "events", error = %e),
    }

    let (event_id, event_slug) = match &events {
        Ok(ev) => (first_event_id(ev), first_event_slug(ev)),
        Err(_) => (None, None),
    };

    if let Some(id) = event_id {
        let request = EventByIdRequest::builder().id(id).build();
        match client.event_by_id(&request).await {
            Ok(e) => info!(
                endpoint = "event_by_id",
                title = e.title.as_deref().unwrap_or("(no title)")
            ),
            Err(e) => error!(endpoint = "event_by_id", error = %e),
        }

        let tags_request = EventTagsRequest::builder().id(id).build();
        match client.event_tags(&tags_request).await {
            Ok(t) => info!(endpoint = "event_tags", count = t.len()),
            Err(e) => error!(endpoint = "event_tags", error = %e),
        }
    } else {
        warn!(endpoint = "event_by_id", reason = "no event ID available");
        warn!(endpoint = "event_tags", reason = "no event ID available");
    }

    if let Some(slug) = event_slug {
        let request = EventBySlugRequest::builder().slug(slug).build();
        match client.event_by_slug(&request).await {
            Ok(e) => info!(
                endpoint = "event_by_slug",
                title = e.title.as_deref().unwrap_or("(no title)")
            ),
            Err(e) => error!(endpoint = "event_by_slug", error = %e),
        }
    } else {
        warn!(
            endpoint = "event_by_slug",
            reason = "no event slug available"
        );
    }

    info!("═══ Markets ═══");
    let markets_request = MarketsRequest::builder().limit(10).build();
    let markets = client.markets(&markets_request).await;
    match &markets {
        Ok(m) => info!(endpoint = "markets", count = m.len()),
        Err(e) => error!(endpoint = "markets", error = %e),
    }

    let (market_id, market_slug) = match &markets {
        Ok(m) => (first_market_id(m), first_market_slug(m)),
        Err(_) => (None, None),
    };

    if let Some(id) = market_id {
        let request = MarketByIdRequest::builder().id(id).build();
        match client.market_by_id(&request).await {
            Ok(m) => info!(
                endpoint = "market_by_id",
                question = m.question.as_deref().unwrap_or("(no question)")
            ),
            Err(e) => error!(endpoint = "market_by_id", error = %e),
        }

        let tags_request = MarketTagsRequest::builder().id(id).build();
        match client.market_tags(&tags_request).await {
            Ok(t) => info!(endpoint = "market_tags", count = t.len()),
            Err(e) => error!(endpoint = "market_tags", error = %e),
        }
    } else {
        warn!(endpoint = "market_by_id", reason = "no market ID available");
        warn!(endpoint = "market_tags", reason = "no market ID available");
    }

    if let Some(slug) = market_slug {
        let request = MarketBySlugRequest::builder().slug(slug).build();
        match client.market_by_slug(&request).await {
            Ok(m) => info!(
                endpoint = "market_by_slug",
                question = m.question.as_deref().unwrap_or("(no question)")
            ),
            Err(e) => error!(endpoint = "market_by_slug", error = %e),
        }
    } else {
        warn!(
            endpoint = "market_by_slug",
            reason = "no market slug available"
        );
    }

    info!("═══ Series ═══");
    let series_request = SeriesListRequest::builder().limit(10).build();
    let series_list = client.series(&series_request).await;
    match &series_list {
        Ok(s) => info!(endpoint = "series", count = s.len()),
        Err(e) => error!(endpoint = "series", error = %e),
    }

    let series_id = match &series_list {
        Ok(s) => first_series_id(s),
        Err(_) => None,
    };

    if let Some(id) = series_id {
        let request = SeriesByIdRequest::builder().id(id).build();
        match client.series_by_id(&request).await {
            Ok(s) => info!(
                endpoint = "series_by_id",
                title = s.title.as_deref().unwrap_or("(no title)")
            ),
            Err(e) => error!(endpoint = "series_by_id", error = %e),
        }
    } else {
        warn!(endpoint = "series_by_id", reason = "no series ID available");
    }

    info!("═══ Comments ═══");

    // Find an event with comments for better demo output
    let event_with_comments = match &events {
        Ok(ev) => find_event_with_comments(&client, ev).await,
        Err(_) => None,
    };

    if let Some((_event, comments)) = event_with_comments {
        info!(endpoint = "comments", count = comments.len());

        if let Some(comment) = comments.first() {
            let body = comment.body.as_deref().unwrap_or("(no body)");
            let truncated: String = if body.chars().count() > 50 {
                format!("{}...", body.chars().take(50).collect::<String>())
            } else {
                body.to_owned()
            };
            info!(endpoint = "comments_by_id", body = %truncated);

            if let Some(addr) = &comment.user_address {
                let request = CommentsByUserAddressRequest::builder()
                    .user_address(addr)
                    .limit(5)
                    .build();
                match client.comments_by_user_address(&request).await {
                    Ok(c) => {
                        info!(endpoint = "comments_by_user_address", count = c.len());
                    }
                    Err(e) => error!(endpoint = "comments_by_user_address", error = %e),
                }
            } else {
                warn!(
                    endpoint = "comments_by_user_address",
                    reason = "no user address on comment"
                );
            }
        }
    } else if let Some(event_id) = events.as_ref().ok().and_then(|ev| first_event_id(ev)) {
        // No event with comments found, but we have events to test the endpoint
        let request = CommentsRequest::builder()
            .parent_entity_type(ParentEntityType::Event)
            .parent_entity_id(event_id)
            .limit(10)
            .build();
        let comments = client.comments(&request).await;
        match &comments {
            Ok(c) => info!(endpoint = "comments", count = c.len()),
            Err(e) => error!(endpoint = "comments", error = %e),
        }

        warn!(
            endpoint = "comments_by_id",
            reason = "no comments found for event"
        );
        warn!(
            endpoint = "comments_by_user_address",
            reason = "no comments found for event"
        );
    } else {
        warn!(endpoint = "comments", reason = "no events available");
        warn!(endpoint = "comments_by_id", reason = "no events available");
        warn!(
            endpoint = "comments_by_user_address",
            reason = "no events available"
        );
    }

    info!("═══ Profiles ═══");
    // Use a known test address for profile testing
    let test_address = "0xa41249c581990c31fb2a0dfc4417ede58e0de774";
    let request = PublicProfileRequest::builder()
        .address(test_address)
        .build();
    match client.public_profile(&request).await {
        Ok(p) => info!(
            endpoint = "public_profile",
            name = p.name.as_deref().unwrap_or("(no name)")
        ),
        Err(e) => error!(endpoint = "public_profile", error = %e),
    }

    info!("═══ Sports ═══");
    match client.sports().await {
        Ok(s) => info!(endpoint = "sports", count = s.len()),
        Err(e) => error!(endpoint = "sports", error = %e),
    }

    match client.sports_market_types().await {
        Ok(r) => info!(
            endpoint = "sports_market_types",
            count = r.market_types.len()
        ),
        Err(e) => error!(endpoint = "sports_market_types", error = %e),
    }

    info!("═══ Teams ═══");
    let teams_request = TeamsRequest::builder().limit(10).build();
    match client.teams(&teams_request).await {
        Ok(t) => info!(endpoint = "teams", count = t.len()),
        Err(e) => error!(endpoint = "teams", error = %e),
    }

    info!("═══ Search ═══");
    let search_request = SearchRequest::builder()
        .q("election")
        .limit_per_type(5)
        .build();
    match client.search(&search_request).await {
        Ok(s) => {
            let event_count = s.events.as_ref().map_or(0, Vec::len);
            let tag_count = s.tags.as_ref().map_or(0, Vec::len);
            info!(
                endpoint = "public_search",
                events = event_count,
                tags = tag_count
            );
        }
        Err(e) => error!(endpoint = "public_search", error = %e),
    }

    info!("═══ Summary ═══");
    info!("All endpoints exercised");
    info!("Endpoints with WARN had no valid IDs to test with");

    Ok(())
}
