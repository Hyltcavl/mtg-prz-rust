// main.rs
mod dl;
use dl::card_parser::fetch_and_parse;

fn main() {
    let url = "https://astraeus.dragonslair.se/product/card-singles/magic?name=reaper+king";
    match fetch_and_parse(url) {
        Ok(results) => {
            for result in results {
                println!("{:#?}", result);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
