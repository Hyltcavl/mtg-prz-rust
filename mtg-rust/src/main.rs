// main.rs
mod dl;
use std::sync::Arc;

use dl::list_links::get_page_count;
use futures::future::join_all;
use tokio::sync::Semaphore;
use tokio::try_join;

#[tokio::main]
async fn main() {
    let base_url = "https://astraeus.dragonslair.se".to_owned();
    let semaphore = Arc::new(Semaphore::new(10));
    let cmcs_available = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16];

    let page_fetches_asyncs = cmcs_available.into_iter().map(|cmc| {
        let semaphore_clone = Arc::clone(&semaphore);
        
        tokio::spawn({
            let value = base_url.clone();
            async move {
                let _permit = semaphore_clone.acquire().await.unwrap(); //Get a permit to run in parallel
                let request_url = format_args!(
                    "{}/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
                    &value, cmc, 0
                )
                .to_string();
                let page_count = get_page_count(&request_url).await.unwrap_or(1);
                println!("cmc: {}, and page_count is {:?}", cmc, page_count);
                (1..=page_count)
                    .map(|count| {
                        format!(
                            "{}/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
                            &value, cmc, count
                        )
                    })
                    .collect::<Vec<String>>()
            }
        })
    });
    // Awaits all of the calls. Can only be done in a for loop, not in a .map
    let mut results = Vec::new();
    for handle in page_fetches_asyncs {
        results.push(handle.await.unwrap());
    }
    println!("{:?}", results);

}
