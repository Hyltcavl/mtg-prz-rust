use crate::cards::card::{CardName, Prices, ScryfallCard, SetName, Vendor, VendorCard};

pub fn reaper_king_card_name() -> CardName {
    CardName::new("Reaper King".to_string()).unwrap()
}

pub fn reaper_king_set_name() -> SetName {
    SetName::new("Shadowmoor".to_string()).unwrap()
}

pub fn reaper_king_set_name_2() -> SetName {
    SetName::new("Mystery booster retail edition foils".to_string()).unwrap()
}

pub fn reaper_king_vendor_card_expensive() -> VendorCard {
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

pub fn reaper_king_vendor_card_cheap() -> VendorCard {
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

pub fn reaper_king_scryfall_card_expensive() -> ScryfallCard {
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

pub fn reaper_king_scryfall_card_cheap() -> ScryfallCard {
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

pub fn lifecraft_c_name() -> CardName {
    CardName::new("Lifecraft Cavalry".to_string()).unwrap()
}

pub fn lifecraft_c_set_name() -> SetName {
    SetName::new("Aether Revolt".to_string()).unwrap()
}

pub fn lifecraft_c_vendor_card() -> VendorCard {
    VendorCard {
        vendor: Vendor::Dragonslair,
        name: lifecraft_c_name(),
        foil: true,
        image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
        extended_art: false,
        prerelease: false,
        showcase: false,
        set: lifecraft_c_set_name(),
        price: 100,
        trade_in_price: 50,
        current_stock: 1,
        max_stock: 2,
    }
}

pub fn lifecraft_c_scryfall_card() -> ScryfallCard {
    ScryfallCard {
        name: lifecraft_c_name(),
        set: lifecraft_c_set_name(),
        image_url: "www.google.com".to_string(),
        prices: Prices {
            eur: Some(1.0),
            eur_foil: Some(2.0),
        },
    }
}
