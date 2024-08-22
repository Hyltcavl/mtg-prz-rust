// main.rs
mod dl;
use dl::{card_parser::fetch_and_parse, list_links::get_page_count};

#[tokio::main]
async fn main() {
    let mut card_urls = Vec::new();
    for x in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16] {
        let request_url = format_args!(
            "https://astraeus.dragonslair.se/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
            x, 0,
        )
        .to_string();
        let page_count = get_page_count(&request_url).await.unwrap();
        println!("x: {}, and page_count is{:?}", x, page_count);

        let pages_to_call = (1 .. page_count+1).collect::<Vec<u32>>()
        .into_iter()
        .map(|count| format_args!(
                "https://astraeus.dragonslair.se/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
                x, count,
            ).to_string()
         )
         .collect::<Vec<String>>();
        card_urls.push(pages_to_call);
    }

    // println!("{:?}", card_urls);
    // let url = "https://astraeus.dragonslair.se/product/card-singles/magic?name=reaper+king";
    // match fetch_and_parse(url) {
    //     Ok(results) => {
    //         for result in results {
    //             println!("{:#?}", result);
    //         }
    //     }
    //     Err(e) => eprintln!("Error: {}", e),
    // }
}
