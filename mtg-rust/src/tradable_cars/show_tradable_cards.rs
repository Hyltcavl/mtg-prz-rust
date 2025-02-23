use super::delver_lense_card::TradeableCard;
use crate::{
    tradable_cars::html_generator::generate_page_content, utils::file_management::save_to_file,
};
use std::fs;

pub fn create_tradable_card_html_page(
    tradable_cards: &Vec<TradeableCard>,
) -> Result<(), Box<dyn std::error::Error>> {
    let html = generate_page_content(&tradable_cards);

    fs::write("/workspaces/mtg-prz-rust/cards.html", html)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::file_management::load_from_json_file;

    #[test]
    fn create_html_page_for_tradable_cards() {
        let mut tradable_cards: Vec<TradeableCard> = load_from_json_file::<Vec<TradeableCard>>(
            "/workspaces/mtg-prz-rust/mtg-rust/tradable_cards_23_02_2025-17-20.json",
        )
        .unwrap();
        tradable_cards.sort_by(|a, b| a.cards_to_trade.cmp(&b.cards_to_trade));

        create_tradable_card_html_page(&tradable_cards).unwrap();
    }
}
