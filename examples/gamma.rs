#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use polymarket_client_sdk::gamma::Client;
use polymarket_client_sdk::gamma::types::{
    ListTeamsRequest, RelatedTagsByIdRequest, RelatedTagsBySlugRequest, TagsRequest,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::default();

    //---- sports endpoints
    println!(
        "teams default -- {:?}",
        client.teams(&ListTeamsRequest::default()).await
    );

    let filtered_request = ListTeamsRequest::builder().limit(5).offset(10).build();
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

    println!("tag_by_id -- {:?}", client.tag_by_id(1, None).await);

    println!(
        "tag_by_slug -- {:?}",
        client.tag_by_slug("politics", None).await
    );

    let request = RelatedTagsByIdRequest::builder().id(1_u64).build();
    println!(
        "tag_relationships_by_id -- {:?}",
        client.tag_relationships_by_id(&request).await
    );

    let request = RelatedTagsBySlugRequest::builder().slug("politics").build();
    println!(
        "tag_relationships_by_slug -- {:?}",
        client.tag_relationships_by_slug(&request).await
    );

    let request = RelatedTagsByIdRequest::builder().id(1_u64).build();
    println!(
        "related_tags_by_id -- {:?}",
        client.related_tags_by_id(&request).await
    );

    let request = RelatedTagsBySlugRequest::builder().slug("politics").build();
    println!(
        "related_tags_by_slug -- {:?}",
        client.related_tags_by_slug(&request).await
    );

    Ok(())
}
