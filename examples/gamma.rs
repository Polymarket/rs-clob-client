#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use polymarket_client_sdk::gamma::Client;
use polymarket_client_sdk::gamma::types::{
    CommentsByIdRequest, CommentsByUserAddressRequest, CommentsRequest, EventByIdRequest,
    EventBySlugRequest, EventTagsRequest, EventsRequest, MarketByIdRequest, MarketBySlugRequest,
    MarketTagsRequest, MarketsRequest, PublicProfileRequest, RelatedTagsByIdRequest,
    RelatedTagsBySlugRequest, SearchRequest, SeriesByIdRequest, SeriesListRequest, TagByIdRequest,
    TagBySlugRequest, TagsRequest, TeamsRequest,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::default();

    //---- health check
    println!("status -- {:?}", client.status().await);

    //---- sports endpoints
    println!(
        "teams default -- {:?}",
        client.teams(&TeamsRequest::default()).await
    );

    let filtered_request = TeamsRequest::builder().limit(5).offset(10).build();
    println!(
        "teams custom -- {:?}",
        client.teams(&filtered_request).await
    );

    println!("sports -- {:?}", client.sports().await);

    println!(
        "sports_market_types -- {:?}",
        client.sports_market_types().await
    );

    //---- tag endpoints
    let request = TagsRequest::builder().build();
    println!("tags -- {:?}", client.tags(&request).await);

    let request = TagByIdRequest::builder().id("1").build();
    println!("tag_by_id -- {:?}", client.tag_by_id(&request).await);

    let request = TagBySlugRequest::builder().slug("politics").build();
    println!("tag_by_slug -- {:?}", client.tag_by_slug(&request).await);

    let request = RelatedTagsByIdRequest::builder().id("1").build();
    println!(
        "related_tags_by_id -- {:?}",
        client.related_tags_by_id(&request).await
    );

    let request = RelatedTagsBySlugRequest::builder().slug("politics").build();
    println!(
        "related_tags_by_slug -- {:?}",
        client.related_tags_by_slug(&request).await
    );

    let request = RelatedTagsByIdRequest::builder().id("1").build();
    println!(
        "tags_related_to_tag_by_id -- {:?}",
        client.tags_related_to_tag_by_id(&request).await
    );

    let request = RelatedTagsBySlugRequest::builder().slug("politics").build();
    println!(
        "tags_related_to_tag_by_slug -- {:?}",
        client.tags_related_to_tag_by_slug(&request).await
    );

    //---- events endpoints
    let request = EventsRequest::builder().active(true).limit(5).build();
    println!("events -- {:?}", client.events(&request).await);

    let request = EventByIdRequest::builder().id("1").build();
    println!("event_by_id -- {:?}", client.event_by_id(&request).await);

    let request = EventBySlugRequest::builder().slug("example-event").build();
    println!(
        "event_by_slug -- {:?}",
        client.event_by_slug(&request).await
    );

    let request = EventTagsRequest::builder().id("1").build();
    println!("event_tags -- {:?}", client.event_tags(&request).await);

    //---- markets endpoints
    let request = MarketsRequest::builder().closed(false).limit(5).build();
    println!("markets -- {:?}", client.markets(&request).await);

    let request = MarketByIdRequest::builder().id("1").build();
    println!("market_by_id -- {:?}", client.market_by_id(&request).await);

    let request = MarketBySlugRequest::builder()
        .slug("example-market")
        .build();
    println!(
        "market_by_slug -- {:?}",
        client.market_by_slug(&request).await
    );

    let request = MarketTagsRequest::builder().id("1").build();
    println!("market_tags -- {:?}", client.market_tags(&request).await);

    //---- series endpoints
    let request = SeriesListRequest::builder().limit(5).build();
    println!("series -- {:?}", client.series(&request).await);

    let request = SeriesByIdRequest::builder().id("1").build();
    println!("series_by_id -- {:?}", client.series_by_id(&request).await);

    //---- comments endpoints
    let request = CommentsRequest::builder().limit(5).build();
    println!("comments -- {:?}", client.comments(&request).await);

    let request = CommentsByIdRequest::builder().id("1").build();
    println!(
        "comments_by_id -- {:?}",
        client.comments_by_id(&request).await
    );

    let request = CommentsByUserAddressRequest::builder()
        .user_address("0x56687bf447db6ffa42ffe2204a05edaa20f55839")
        .limit(5)
        .build();
    println!(
        "comments_by_user_address -- {:?}",
        client.comments_by_user_address(&request).await
    );

    //---- profile endpoints
    let request = PublicProfileRequest::builder()
        .address("0x56687bf447db6ffa42ffe2204a05edaa20f55839")
        .build();
    println!(
        "public_profile -- {:?}",
        client.public_profile(&request).await
    );

    //---- search endpoints
    let request = SearchRequest::builder().q("bitcoin").build();
    println!("search -- {:?}", client.search(&request).await);

    Ok(())
}
