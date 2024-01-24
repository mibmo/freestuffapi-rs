use freestuffapi::Client;
use futures::stream::{self, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("FSA_API").expect("FSA_API envvar not set");
    let client = Client::builder().key(&api_key).build()?;

    stream::iter(client.game_list("free").await?.chunks(5).take(3))
        .then(|ids| client.game_details(&ids))
        .flat_map_unordered(None, |map| {
            let values = map.unwrap_or_default().into_values();
            stream::iter(values)
        })
        .for_each(|info| async move {
            dbg!(info);
        })
        .await;

    Ok(())
}
