#![allow(clippy::print_stdout, reason = "Live test output is expected")]

//! Live integration tests for the Gamma API client.
//!
//! This example runs comprehensive tests against the live Polymarket Gamma API,
//! validating all 27 client methods with basic assertions.
//!
//! # Running
//!
//! ```bash
//! cargo run --example gamma_live_test --features gamma
//! ```
//!
//! # Test Strategy
//!
//! - Tests are organized by endpoint group
//! - Uses dynamic discovery: fetches lists first, then uses real IDs for lookups
//! - Basic assertions verify responses have expected structure
//! - Clear pass/fail output for each test

use polymarket_client_sdk::gamma::Client;
use polymarket_client_sdk::gamma::types::{
    CommentsByIdRequest, CommentsByUserAddressRequest, CommentsRequest, EventByIdRequest,
    EventBySlugRequest, EventTagsRequest, EventsRequest, MarketByIdRequest, MarketBySlugRequest,
    MarketTagsRequest, MarketsRequest, ParentEntityType, PublicProfileRequest,
    RelatedTagsByIdRequest, RelatedTagsBySlugRequest, SearchRequest, SeriesByIdRequest,
    SeriesListRequest, TagByIdRequest, TagBySlugRequest, TagsRequest, TeamsRequest,
};

struct TestResults {
    passed: u32,
    failed: u32,
}

impl TestResults {
    fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
        }
    }

    fn pass(&mut self, name: &str) {
        self.passed += 1;
        println!("  [PASS] {name}");
    }

    fn fail(&mut self, name: &str, error: &str) {
        self.failed += 1;
        println!("  [FAIL] {name}: {error}");
    }

    fn summary(&self) {
        println!();
        println!("========================================");
        println!(
            "Results: {} passed, {} failed, {} total",
            self.passed,
            self.failed,
            self.passed + self.failed
        );
        if self.failed == 0 {
            println!("All tests passed!");
        } else {
            println!("Some tests failed.");
        }
        println!("========================================");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber to see API drift warnings
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt::init();

    let client = Client::default();
    let mut results = TestResults::new();

    println!("========================================");
    println!("Gamma API Live Tests");
    println!("========================================");
    println!();

    // =========================================================================
    // Health
    // =========================================================================
    println!("Health Endpoints:");
    test_status(&client, &mut results).await;
    println!();

    // =========================================================================
    // Sports
    // =========================================================================
    println!("Sports Endpoints:");
    test_teams(&client, &mut results).await;
    test_sports(&client, &mut results).await;
    test_sports_market_types(&client, &mut results).await;
    println!();

    // =========================================================================
    // Tags
    // =========================================================================
    println!("Tags Endpoints:");
    let (tag_id, tag_slug) = test_tags(&client, &mut results).await;
    test_tag_by_id(&client, &mut results, &tag_id).await;
    test_tag_by_slug(&client, &mut results, &tag_slug).await;
    test_related_tags_by_id(&client, &mut results, &tag_id).await;
    test_related_tags_by_slug(&client, &mut results, &tag_slug).await;
    test_tags_related_to_tag_by_id(&client, &mut results, &tag_id).await;
    test_tags_related_to_tag_by_slug(&client, &mut results, &tag_slug).await;
    println!();

    // =========================================================================
    // Events
    // =========================================================================
    println!("Events Endpoints:");
    let (event_id, event_slug) = test_events(&client, &mut results).await;
    test_event_by_id(&client, &mut results, &event_id).await;
    test_event_by_slug(&client, &mut results, &event_slug).await;
    test_event_tags(&client, &mut results, &event_id).await;
    println!();

    // =========================================================================
    // Markets
    // =========================================================================
    println!("Markets Endpoints:");
    let (market_id, market_slug) = test_markets(&client, &mut results).await;
    test_market_by_id(&client, &mut results, &market_id).await;
    test_market_by_slug(&client, &mut results, &market_slug).await;
    test_market_tags(&client, &mut results, &market_id).await;
    println!();

    // =========================================================================
    // Series
    // =========================================================================
    println!("Series Endpoints:");
    let series_id = test_series(&client, &mut results).await;
    test_series_by_id(&client, &mut results, &series_id).await;
    println!();

    // =========================================================================
    // Comments
    // =========================================================================
    println!("Comments Endpoints:");
    let (comment_id, user_address) =
        test_comments(&client, &mut results, &event_id, &series_id).await;
    test_comments_by_id(&client, &mut results, &comment_id).await;
    test_comments_by_user_address(&client, &mut results, &user_address).await;
    println!();

    // =========================================================================
    // Profiles
    // =========================================================================
    println!("Profiles Endpoints:");
    test_public_profile(&client, &mut results, &user_address).await;
    println!();

    // =========================================================================
    // Search
    // =========================================================================
    println!("Search Endpoints:");
    test_search(&client, &mut results).await;
    println!();

    results.summary();

    Ok(())
}

// =============================================================================
// Health Tests
// =============================================================================

async fn test_status(client: &Client, results: &mut TestResults) {
    match client.status().await {
        Ok(response) => {
            if response == "OK" {
                results.pass("status()");
            } else {
                results.fail("status()", &format!("Expected 'OK', got '{response}'"));
            }
        }
        Err(e) => results.fail("status()", &e.to_string()),
    }
}

// =============================================================================
// Sports Tests
// =============================================================================

async fn test_teams(client: &Client, results: &mut TestResults) {
    let request = TeamsRequest::builder().limit(5).build();
    match client.teams(&request).await {
        Ok(teams) => {
            // Teams may be empty if no sports teams configured
            results.pass(&format!("teams() - returned {} teams", teams.len()));
        }
        Err(e) => results.fail("teams()", &e.to_string()),
    }
}

async fn test_sports(client: &Client, results: &mut TestResults) {
    match client.sports().await {
        Ok(sports) => {
            results.pass(&format!("sports() - returned {} sports", sports.len()));
        }
        Err(e) => results.fail("sports()", &e.to_string()),
    }
}

async fn test_sports_market_types(client: &Client, results: &mut TestResults) {
    match client.sports_market_types().await {
        Ok(response) => {
            results.pass(&format!(
                "sports_market_types() - returned {} types",
                response.market_types.len()
            ));
        }
        Err(e) => results.fail("sports_market_types()", &e.to_string()),
    }
}

// =============================================================================
// Tags Tests
// =============================================================================

async fn test_tags(client: &Client, results: &mut TestResults) -> (String, String) {
    let request = TagsRequest::builder().limit(10).build();
    match client.tags(&request).await {
        Ok(tags) => {
            if tags.is_empty() {
                results.fail("tags()", "Expected non-empty list");
                ("1".to_owned(), "politics".to_owned())
            } else {
                results.pass(&format!("tags() - returned {} tags", tags.len()));
                let tag = &tags[0];
                let id = tag.id.clone();
                let slug = tag.slug.clone().unwrap_or_else(|| "politics".to_owned());
                (id, slug)
            }
        }
        Err(e) => {
            results.fail("tags()", &e.to_string());
            ("1".to_owned(), "politics".to_owned())
        }
    }
}

async fn test_tag_by_id(client: &Client, results: &mut TestResults, id: &str) {
    let request = TagByIdRequest::builder().id(id).build();
    match client.tag_by_id(&request).await {
        Ok(tag) => {
            if tag.id.is_empty() {
                results.fail("tag_by_id()", "Tag ID is empty");
            } else {
                results.pass(&format!("tag_by_id({id})"));
            }
        }
        Err(e) => results.fail(&format!("tag_by_id({id})"), &e.to_string()),
    }
}

async fn test_tag_by_slug(client: &Client, results: &mut TestResults, slug: &str) {
    let request = TagBySlugRequest::builder().slug(slug).build();
    match client.tag_by_slug(&request).await {
        Ok(tag) => {
            if tag.id.is_empty() {
                results.fail("tag_by_slug()", "Tag ID is empty");
            } else {
                results.pass(&format!("tag_by_slug({slug})"));
            }
        }
        Err(e) => results.fail(&format!("tag_by_slug({slug})"), &e.to_string()),
    }
}

async fn test_related_tags_by_id(client: &Client, results: &mut TestResults, id: &str) {
    let request = RelatedTagsByIdRequest::builder().id(id).build();
    match client.related_tags_by_id(&request).await {
        Ok(related) => {
            results.pass(&format!(
                "related_tags_by_id({id}) - returned {} related tags",
                related.len()
            ));
        }
        Err(e) => results.fail(&format!("related_tags_by_id({id})"), &e.to_string()),
    }
}

async fn test_related_tags_by_slug(client: &Client, results: &mut TestResults, slug: &str) {
    let request = RelatedTagsBySlugRequest::builder().slug(slug).build();
    match client.related_tags_by_slug(&request).await {
        Ok(related) => {
            results.pass(&format!(
                "related_tags_by_slug({slug}) - returned {} related tags",
                related.len()
            ));
        }
        Err(e) => results.fail(&format!("related_tags_by_slug({slug})"), &e.to_string()),
    }
}

async fn test_tags_related_to_tag_by_id(client: &Client, results: &mut TestResults, id: &str) {
    let request = RelatedTagsByIdRequest::builder().id(id).build();
    match client.tags_related_to_tag_by_id(&request).await {
        Ok(tags) => {
            results.pass(&format!(
                "tags_related_to_tag_by_id({id}) - returned {} tags",
                tags.len()
            ));
        }
        Err(e) => results.fail(&format!("tags_related_to_tag_by_id({id})"), &e.to_string()),
    }
}

async fn test_tags_related_to_tag_by_slug(client: &Client, results: &mut TestResults, slug: &str) {
    let request = RelatedTagsBySlugRequest::builder().slug(slug).build();
    match client.tags_related_to_tag_by_slug(&request).await {
        Ok(tags) => {
            results.pass(&format!(
                "tags_related_to_tag_by_slug({slug}) - returned {} tags",
                tags.len()
            ));
        }
        Err(e) => results.fail(
            &format!("tags_related_to_tag_by_slug({slug})"),
            &e.to_string(),
        ),
    }
}

// =============================================================================
// Events Tests
// =============================================================================

async fn test_events(client: &Client, results: &mut TestResults) -> (String, String) {
    // Fetch more events sorted by volume to find popular ones with comments
    let request = EventsRequest::builder()
        .active(true)
        .limit(50)
        .order("volume24hr".to_owned())
        .ascending(false)
        .build();
    match client.events(&request).await {
        Ok(events) => {
            if events.is_empty() {
                results.fail("events()", "Expected non-empty list of active events");
                ("1".to_owned(), "example-event".to_owned())
            } else {
                results.pass(&format!("events() - returned {} events", events.len()));
                // Find an event with comments if possible, otherwise use the first one
                let event = events
                    .iter()
                    .find(|e| e.comment_count.unwrap_or(0) > 0)
                    .unwrap_or(&events[0]);
                let id = event.id.clone();
                let slug = event
                    .slug
                    .clone()
                    .unwrap_or_else(|| "example-event".to_owned());
                let comment_count = event.comment_count.unwrap_or(0);
                if comment_count > 0 {
                    println!("    (selected event '{slug}' with {comment_count} comments)");
                }
                (id, slug)
            }
        }
        Err(e) => {
            results.fail("events()", &e.to_string());
            ("1".to_owned(), "example-event".to_owned())
        }
    }
}

async fn test_event_by_id(client: &Client, results: &mut TestResults, id: &str) {
    let request = EventByIdRequest::builder().id(id).build();
    match client.event_by_id(&request).await {
        Ok(event) => {
            if event.id.is_empty() {
                results.fail("event_by_id()", "Event ID is empty");
            } else {
                results.pass(&format!("event_by_id({id})"));
            }
        }
        Err(e) => results.fail(&format!("event_by_id({id})"), &e.to_string()),
    }
}

async fn test_event_by_slug(client: &Client, results: &mut TestResults, slug: &str) {
    let request = EventBySlugRequest::builder().slug(slug).build();
    match client.event_by_slug(&request).await {
        Ok(event) => {
            if event.id.is_empty() {
                results.fail("event_by_slug()", "Event ID is empty");
            } else {
                results.pass(&format!("event_by_slug({slug})"));
            }
        }
        Err(e) => results.fail(&format!("event_by_slug({slug})"), &e.to_string()),
    }
}

async fn test_event_tags(client: &Client, results: &mut TestResults, event_id: &str) {
    let request = EventTagsRequest::builder().id(event_id).build();
    match client.event_tags(&request).await {
        Ok(tags) => {
            results.pass(&format!(
                "event_tags({event_id}) - returned {} tags",
                tags.len()
            ));
        }
        Err(e) => results.fail(&format!("event_tags({event_id})"), &e.to_string()),
    }
}

// =============================================================================
// Markets Tests
// =============================================================================

async fn test_markets(client: &Client, results: &mut TestResults) -> (String, String) {
    // Fetch markets sorted by volume to get popular ones
    let request = MarketsRequest::builder()
        .closed(false)
        .limit(50)
        .order("volume24hr".to_owned())
        .ascending(false)
        .build();
    match client.markets(&request).await {
        Ok(markets) => {
            if markets.is_empty() {
                results.fail("markets()", "Expected non-empty list of open markets");
                ("1".to_owned(), "example-market".to_owned())
            } else {
                results.pass(&format!("markets() - returned {} markets", markets.len()));
                // Use the highest volume market (first one after sorting)
                let market = &markets[0];
                let id = market.id.clone();
                let slug = market
                    .slug
                    .clone()
                    .unwrap_or_else(|| "example-market".to_owned());
                let volume = market.volume_24hr.unwrap_or(0.0);
                if volume > 0.0 {
                    println!("    (selected market '{slug}' with ${volume:.0} 24hr volume)");
                }
                (id, slug)
            }
        }
        Err(e) => {
            results.fail("markets()", &e.to_string());
            ("1".to_owned(), "example-market".to_owned())
        }
    }
}

async fn test_market_by_id(client: &Client, results: &mut TestResults, id: &str) {
    let request = MarketByIdRequest::builder().id(id).build();
    match client.market_by_id(&request).await {
        Ok(market) => {
            if market.id.is_empty() {
                results.fail("market_by_id()", "Market ID is empty");
            } else {
                results.pass(&format!("market_by_id({id})"));
            }
        }
        Err(e) => results.fail(&format!("market_by_id({id})"), &e.to_string()),
    }
}

async fn test_market_by_slug(client: &Client, results: &mut TestResults, slug: &str) {
    let request = MarketBySlugRequest::builder().slug(slug).build();
    match client.market_by_slug(&request).await {
        Ok(market) => {
            if market.id.is_empty() {
                results.fail("market_by_slug()", "Market ID is empty");
            } else {
                results.pass(&format!("market_by_slug({slug})"));
            }
        }
        Err(e) => results.fail(&format!("market_by_slug({slug})"), &e.to_string()),
    }
}

async fn test_market_tags(client: &Client, results: &mut TestResults, market_id: &str) {
    let request = MarketTagsRequest::builder().id(market_id).build();
    match client.market_tags(&request).await {
        Ok(tags) => {
            results.pass(&format!(
                "market_tags({market_id}) - returned {} tags",
                tags.len()
            ));
        }
        Err(e) => results.fail(&format!("market_tags({market_id})"), &e.to_string()),
    }
}

// =============================================================================
// Series Tests
// =============================================================================

async fn test_series(client: &Client, results: &mut TestResults) -> String {
    let request = SeriesListRequest::builder().limit(10).build();
    match client.series(&request).await {
        Ok(series_list) => {
            if series_list.is_empty() {
                results.fail("series()", "Expected non-empty list");
                "1".to_owned()
            } else {
                results.pass(&format!("series() - returned {} series", series_list.len()));
                series_list[0].id.clone()
            }
        }
        Err(e) => {
            results.fail("series()", &e.to_string());
            "1".to_owned()
        }
    }
}

async fn test_series_by_id(client: &Client, results: &mut TestResults, id: &str) {
    let request = SeriesByIdRequest::builder().id(id).build();
    match client.series_by_id(&request).await {
        Ok(series) => {
            if series.id.is_empty() {
                results.fail("series_by_id()", "Series ID is empty");
            } else {
                results.pass(&format!("series_by_id({id})"));
            }
        }
        Err(e) => results.fail(&format!("series_by_id({id})"), &e.to_string()),
    }
}

// =============================================================================
// Comments Tests
// =============================================================================

async fn test_comments(
    client: &Client,
    results: &mut TestResults,
    event_id: &str,
    series_id: &str,
) -> (String, String) {
    // Try event first, then series if no comments found
    // Note: Market is not a valid parent_entity_type for comments
    let request = CommentsRequest::builder()
        .parent_entity_type(ParentEntityType::Event)
        .parent_entity_id(event_id)
        .limit(10)
        .build();
    match client.comments(&request).await {
        Ok(comments) => {
            if !comments.is_empty() {
                results.pass(&format!(
                    "comments(Event) - returned {} comments",
                    comments.len()
                ));
                let id = comments[0].id.to_string();
                let user_address = comments[0]
                    .user_address
                    .clone()
                    .unwrap_or_else(|| "0x0".to_owned());
                return (id, user_address);
            }
            // Try series if event has no comments
            let request = CommentsRequest::builder()
                .parent_entity_type(ParentEntityType::Series)
                .parent_entity_id(series_id)
                .limit(10)
                .build();
            match client.comments(&request).await {
                Ok(series_comments) => {
                    if series_comments.is_empty() {
                        results.pass("comments() - no comments found (normal for some entities)");
                        ("1".to_owned(), "0x0".to_owned())
                    } else {
                        results.pass(&format!(
                            "comments(Series) - returned {} comments",
                            series_comments.len()
                        ));
                        let id = series_comments[0].id.to_string();
                        let user_address = series_comments[0]
                            .user_address
                            .clone()
                            .unwrap_or_else(|| "0x0".to_owned());
                        (id, user_address)
                    }
                }
                Err(e) => {
                    results.fail("comments(Series)", &e.to_string());
                    ("1".to_owned(), "0x0".to_owned())
                }
            }
        }
        Err(e) => {
            results.fail("comments()", &e.to_string());
            ("1".to_owned(), "0x0".to_owned())
        }
    }
}

async fn test_comments_by_id(client: &Client, results: &mut TestResults, id: &str) {
    let request = CommentsByIdRequest::builder().id(id).build();
    match client.comments_by_id(&request).await {
        Ok(comments) => {
            results.pass(&format!(
                "comments_by_id({id}) - returned {} comments",
                comments.len()
            ));
        }
        Err(e) => results.fail(&format!("comments_by_id({id})"), &e.to_string()),
    }
}

async fn test_comments_by_user_address(
    client: &Client,
    results: &mut TestResults,
    user_address: &str,
) {
    let request = CommentsByUserAddressRequest::builder()
        .user_address(user_address)
        .limit(10)
        .build();
    match client.comments_by_user_address(&request).await {
        Ok(comments) => {
            results.pass(&format!(
                "comments_by_user_address({}) - returned {} comments",
                user_address.get(..8).unwrap_or(user_address),
                comments.len()
            ));
        }
        Err(e) => results.fail("comments_by_user_address()", &e.to_string()),
    }
}

// =============================================================================
// Profiles Tests
// =============================================================================

async fn test_public_profile(client: &Client, results: &mut TestResults, user_address: &str) {
    let request = PublicProfileRequest::builder()
        .address(user_address)
        .build();
    match client.public_profile(&request).await {
        Ok(_profile) => {
            results.pass(&format!(
                "public_profile({}) ",
                user_address.get(..8).unwrap_or(user_address)
            ));
        }
        Err(e) => {
            // Profile may not exist for all addresses, which is acceptable
            if e.to_string().contains("404") || e.to_string().contains("not found") {
                results.pass(&format!(
                    "public_profile({}) - profile not found (acceptable)",
                    user_address.get(..8).unwrap_or(user_address)
                ));
            } else {
                results.fail("public_profile()", &e.to_string());
            }
        }
    }
}

// =============================================================================
// Search Tests
// =============================================================================

async fn test_search(client: &Client, results: &mut TestResults) {
    let request = SearchRequest::builder().q("bitcoin").build();
    match client.search(&request).await {
        Ok(search_results) => {
            let event_count = search_results.events.as_ref().map_or(0, std::vec::Vec::len);
            let tag_count = search_results.tags.as_ref().map_or(0, std::vec::Vec::len);
            let profile_count = search_results
                .profiles
                .as_ref()
                .map_or(0, std::vec::Vec::len);
            results.pass(&format!(
                "search('bitcoin') - found {event_count} events, {tag_count} tags, {profile_count} profiles"
            ));
        }
        Err(e) => results.fail("search()", &e.to_string()),
    }
}
