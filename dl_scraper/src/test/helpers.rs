use crate::cards::currency::Currency;
use crate::cards::price::Price;
use crate::cards::{
    cardname::CardName,
    scryfallcard::{Prices, ScryfallCard},
    setname::SetName,
    vendor::Vendor,
    vendorcard::VendorCard,
};

pub fn reaper_king_card_name() -> CardName {
    CardName::new("Reaper King".to_string()).unwrap()
}

pub fn reaper_king_set_name() -> SetName {
    SetName::new("Shadowmoor".to_string()).unwrap()
}

pub fn reaper_king_set_name_2() -> SetName {
    SetName::new("Mystery booster retail edition foils".to_string()).unwrap()
}

// pub fn counterspell_forth_e() -> VendorCard {
//     VendorCard {
//         vendor: Vendor::Dragonslair,
//         name: CardName::new("Counterspell".to_string()).unwrap(),
//         foil: false,
//         image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
//         extended_art: false,
//         prerelease: false,
//         showcase: false,
//         set: SetName::new("Magic 25".to_string()).unwrap(),
//         price: 100,
//         trade_in_price: 50,
//         current_stock: 6,
//         max_stock: 4,
//     }
// }

// pub fn counterspell_ice_age() -> VendorCard {
//     VendorCard {
//         vendor: Vendor::Dragonslair,
//         name: CardName::new("Counterspell".to_string()).unwrap(),
//         foil: false,
//         image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
//         extended_art: false,
//         prerelease: false,
//         showcase: false,
//         set: SetName::new("Ice Age".to_string()).unwrap(),
//         price: 100,
//         trade_in_price: 50,
//         current_stock: 2,
//         max_stock: 4,
//     }
// }

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
            eur: Some(Price::new(1.0, Currency::EUR)),
            eur_foil: Some(Price::new(2.0, Currency::EUR)),
        },
    }
}

pub fn reaper_king_scryfall_card_cheap() -> ScryfallCard {
    ScryfallCard {
        name: reaper_king_card_name(),
        set: reaper_king_set_name_2(),
        image_url: "www.google.com".to_string(),
        prices: Prices {
            eur: Some(Price::new(0.3, Currency::EUR)),
            eur_foil: Some(Price::new(1.0, Currency::EUR)),
        },
    }
}

pub fn lifecraft_c_name() -> CardName {
    CardName::new("Lifecraft Cavalry".to_string()).unwrap()
}

pub fn cardname_sunken_ruins() -> CardName {
    CardName::new("Sunken Ruins (Foil)".to_string()).unwrap()
}

pub fn setname_sunken_ruins() -> SetName {
    SetName::new("Double Masters".to_string()).unwrap()
}

pub fn vendor_card_sunken_ruins_foil() -> VendorCard {
    VendorCard {
        vendor: Vendor::Dragonslair,
        name: cardname_sunken_ruins(),
        foil: true,
        image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
        extended_art: false,
        prerelease: false,
        showcase: false,
        set: setname_sunken_ruins(),
        price: 180,
        trade_in_price: 100,
        current_stock: 1,
        max_stock: 1,
    }
}

pub fn scryfall_card_sunken_ruins() -> ScryfallCard {
    ScryfallCard {
        name: cardname_sunken_ruins(),
        set: setname_sunken_ruins(),
        image_url: "www.google.com".to_string(),
        prices: Prices {
            eur: Some(Price::new(17.71, Currency::EUR)),
            eur_foil: Some(Price::new(20.65, Currency::EUR)),
        },
    }
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
            eur: Some(Price::new(1.0, Currency::EUR)),
            eur_foil: Some(Price::new(2.0, Currency::EUR)),
        },
    }
}

pub fn lifecraft_scryfall_card_no_price() -> ScryfallCard {
    ScryfallCard {
        name: lifecraft_c_name(),
        set: SetName::new("random".to_string()).unwrap(),
        image_url: "www.google.com".to_string(),
        prices: Prices {
            eur: None,
            eur_foil: None,
        },
    }
}
