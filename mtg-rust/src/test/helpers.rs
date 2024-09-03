use crate::cards::card::{CardName, Prices, ScryfallCard, SetName, Vendor, VendorCard};

pub static REAPER_KING_STRING: &str = "Reaper King";

pub fn reaper_king_card_name() -> CardName {
    CardName::new(REAPER_KING_STRING.to_string()).unwrap()
}

pub fn reaper_king_set_name() -> SetName {
    SetName::new("Shadowmoor".to_string()).unwrap()
}

pub fn reaper_king_set_name_2() -> SetName {
    SetName::new("Mystery booster retail edition foils".to_string()).unwrap()
}

pub fn reaper_king_vendor_card() -> VendorCard {
    VendorCard {
        vendor: Vendor::Dragonslair,
        name: reaper_king_card_name(),
        foil: false,
        image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
        extended_art: false,
        prerelease: false,
        showcase: false,
        set: reaper_king_set_name(),
        price: 100,
        trade_in_price: 50,
        current_stock: 1,
        max_stock: 2,
    }
}

pub fn reaper_king_vendor_card_2() -> VendorCard {
    VendorCard {
        vendor: Vendor::Dragonslair,
        name: reaper_king_card_name(),
        foil: false,
        image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
        extended_art: false,
        prerelease: false,
        showcase: false,
        set: reaper_king_set_name_2(),
        price: 50,
        trade_in_price: 40,
        current_stock: 1,
        max_stock: 2,
    }
}

pub fn reaper_king_vendor_card_foil() -> VendorCard {
    VendorCard {
        vendor: Vendor::Dragonslair,
        name: reaper_king_card_name(),
        foil: true,
        image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
        extended_art: false,
        prerelease: false,
        showcase: false,
        set: reaper_king_set_name(),
        price: 200,
        trade_in_price: 100,
        current_stock: 1,
        max_stock: 2,
    }
}

pub fn reaper_king_scryfall_card() -> ScryfallCard {
    ScryfallCard {
        name: reaper_king_card_name(),
        set: reaper_king_set_name(),
        image_url: "www.google.com".to_string(),
        prices: Prices {
            eur: Some(1.0),
            eur_foil: Some(2.0),
        },
    }
}

pub fn reaper_king_scryfall_card_2() -> ScryfallCard {
    ScryfallCard {
        name: reaper_king_card_name(),
        set: reaper_king_set_name_2(),
        image_url: "www.google.com".to_string(),
        prices: Prices {
            eur: Some(0.5),
            eur_foil: Some(1.0),
        },
    }
}
