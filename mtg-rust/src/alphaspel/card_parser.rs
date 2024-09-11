use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use std::{collections::HashMap, sync::Arc};

use chrono::Local;
use futures::future::{err, join_all};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::json;
use tokio::sync::Semaphore;

use crate::{
    cards::card::{CardName, SetName, Vendor, VendorCard},
    utils::{file_management::save_to_json_file, string_manipulators::date_time_as_string},
};

const PROMO_PATTERNS: [&str; 4] = [
    r"(?i)\(Promo\)",
    r"(?i)\(promo\)",
    r"(?i)\(Prerelease\)",
    r"(?i)\(prerelease\)",
];

fn create_regex_patterns(patterns: &[&str]) -> Result<Vec<Regex>, Box<dyn Error>> {
    patterns
        .iter()
        .map(|&p| Regex::new(p).map_err(|e| Box::new(e) as Box<dyn Error>)) // Convert regex::Error to Box<dyn Error>
        .collect()
}

fn get_card_information(product: scraper::ElementRef) -> Result<VendorCard, Box<dyn Error>> {
    let in_stock = product
        .select(&Selector::parse(".stock").unwrap())
        .next()
        .ok_or("No stock information found")?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    let stock = if in_stock == "Slutsåld" {
        return Err("Card is Slutsåld".into());
    } else {
        in_stock.replace("i butiken", "").trim().parse::<i8>()?
    };

    let image_url: String = product
        .select(&Selector::parse("img.img-responsive.center-block").unwrap())
        .filter_map(|element| element.value().attr("src"))
        .map(|href| href.to_string())
        .collect();

    let image_url = format!("https://alphaspel.se{}", image_url.replace("\n", "").trim());

    let product_name = product
        .select(&Selector::parse(".product-name").unwrap())
        .next()
        .ok_or("No product name found")?
        .text()
        .collect::<String>();

    let product_name = product_name
        .replace("\n", "")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    let promo_patterns = create_regex_patterns(&PROMO_PATTERNS)?;
    let prerelease = promo_patterns
        .iter()
        .any(|pattern| pattern.is_match(&product_name));

    let alternative_art = product_name.contains("(alternative art)");

    if product_name.to_lowercase().contains("(italiensk)")
        || product_name.to_lowercase().contains("(tysk)")
        || product_name.to_lowercase().contains("(rysk)")
    {
        return Err("Card is not english".into());
    }

    let set_name: Vec<&str> = product_name.trim().split(":").collect();

    let (raw_name, set) = match set_name.len() {
        5 => (
            set_name[4],
            format!("{}:{}", set_name[2], set_name[3])
                .trim()
                .to_string(),
        ),
        3 => (set_name[2], set_name[1].trim().to_string()),
        _ => return Err("Unexpected set name format".into()),
    };

    if Regex::new(r"Token")?.is_match(raw_name) {
        return Err("Card is a token".into());
    }

    let price = product
        .select(&Selector::parse(".price.text-success").unwrap())
        .next()
        .ok_or("No price found")?
        .text()
        .collect::<String>();
    let price: i32 = Regex::new(r"\d+")?
        .find(&price)
        .ok_or("No price found")?
        .as_str()
        .parse()?;

    let foil = Regex::new(r"\(Foil\)")?.is_match(raw_name)
        || Regex::new(r"\(Etched Foil\)")?.is_match(raw_name)
        || Regex::new(r"\(Foil Etched\)")?.is_match(raw_name);

    let raw_name = raw_name.replace("(Begagnad)", "").trim().to_string();
    let mut name = Regex::new(r"\([^()]*\)")?
        .replace_all(&raw_name, "")
        .to_string();
    name = name
        .replace("v.2", "")
        .replace("V.2", "")
        .replace("v.1", "")
        .replace("v.3", "")
        .replace("v.4", "")
        .trim()
        .to_string();
    name = Regex::new(r"\b(\w+)\s/\s(\w+)\b")?
        .replace_all(&name, "$1 // $2")
        .to_string();

    let prefixes = [
        "Commander 2016 ",
        "Conflux ",
        "Eventide ",
        "Shadowmoor ",
        "Planechase card bundle ",
    ];
    for prefix in prefixes.iter() {
        name = name.strip_prefix(prefix).unwrap_or(&name).to_string();
    }
    let name = CardName::new(name)?;
    let set = SetName::new(set)?;

    Ok(VendorCard {
        name,
        vendor: Vendor::Alphaspel,
        foil,
        image_url: image_url,
        extended_art: alternative_art,
        prerelease: prerelease,
        showcase: false,
        set,
        price,
        trade_in_price: 0,
        current_stock: stock,
        max_stock: 3,
    })
}

// Returns a list of all available set pages which has cards.
async fn get_all_card_pages(base_url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let sets_page = reqwest::get(format!("{base_url}/1978-mtg-loskort/"))
        .await?
        .text()
        .await?;

    let document = Html::parse_document(&sets_page);
    // let selector = Selector::parse(".nav.nav-list a").unwrap();
    let selector = Selector::parse(".categories.row h4.text-center a").unwrap();

    let sets_links: Vec<String> = document
        .select(&selector)
        .filter_map(|element| element.value().attr("href"))
        .map(|href| href.to_string())
        .collect();

    Ok(sets_links)
}
// shouldb me https://alphaspel.se
pub async fn download_alpha_cards(base_url: &str) -> Result<String, Box<dyn Error>> {
    let start_time = chrono::prelude::Local::now();
    let start_time_as_string = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    log::info!("Alphaspel scan started at: {}", start_time_as_string,);

    let pages = get_all_card_pages(base_url).await?;

    let mut grouped_cards: HashMap<String, Vec<VendorCard>> = HashMap::new();

    let semaphore = Arc::new(Semaphore::new(20));

    // let mut links_and_page_numbers: HashMap<String, u32> = HashMap::new();
    // let mut index = 0;
    let futures = pages.into_iter().map(|set_href| {
        let semaphore_clone = Arc::clone(&semaphore);
        async move {
            let _permit = semaphore_clone.acquire().await.unwrap(); //Get a permit to run in parallel

            let link = format!("{base_url}{set_href}?order_by=stock_a&ordering=desc&page=1");
            let set_initial_page = reqwest::get(&link).await.unwrap().text().await.unwrap();
            let document = Html::parse_document(&set_initial_page);
            let selector = Selector::parse("ul.pagination li").unwrap();

            let mut max_page = 0;
            for element in document.select(&selector) {
                if let Ok(num) = element.text().collect::<String>().trim().parse::<u32>() {
                    if num > max_page {
                        max_page = num;
                    }
                }
            }

            (set_href, max_page)
        }
    });

    let links_and_page_numbers: HashMap<String, u32> =
        join_all(futures).await.into_iter().collect();

    log::info!("{:?}", links_and_page_numbers);
    let futures = links_and_page_numbers.iter().map(|(set_href, pages)| {
        let semaphore_clone = Arc::clone(&semaphore);
        async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            let mut cards = Vec::new();

            for x in 1..=*pages as i32 {
                let link = format!("{base_url}{set_href}?order_by=stock_a&ordering=desc&page={x}");
                log::info!("Processing link {}", &link);
                let set_page = reqwest::get(&link)
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("Error fetching page: {}, with error: {}", &link, e);
                        return "".to_string();
                    });
                let document = Html::parse_document(&set_page);
                let product_selector = &Selector::parse(".products.row .product").unwrap();
                let products = document.select(product_selector);

                for product in products {
                    if let Ok(card) = get_card_information(product) {
                        cards.push(card);
                    }
                }
            }
            cards
        }
    });

    let cards: Vec<VendorCard> = join_all(futures).await.into_iter().flatten().collect();

    for card in &cards {
        grouped_cards
            .entry(card.name.almost_raw.clone())
            .or_insert_with(Vec::new)
            .push(card.clone())
    }

    let alpha_cards_path = format!(
        "alphaspel_cards/alphaspel_cards_{}.json",
        date_time_as_string(None, None)
    );

    save_to_json_file(&alpha_cards_path, &grouped_cards)?;

    // let duration = start_time.elapsed();
    let end_time = chrono::prelude::Local::now();
    log::info!(
        "Alphaspel scan started at: {}. Finished at: {}. Took: {} seconds and with {} cards on dl_cards_path: {}",
        start_time_as_string,
        end_time,
        (end_time - start_time).num_seconds(),
        cards.len(),
        alpha_cards_path
    );

    Ok(alpha_cards_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tokio;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn test_get_all_card_pages() {
        init();

        let html_content = fs::read_to_string(
            "/workspaces/mtg-prz-rust/mtg-rust/src/alphaspel/alphaspel_starter_page.html",
        )
        .unwrap();

        let mut server = std::thread::spawn(|| mockito::Server::new())
            .join()
            .unwrap();
        let url = server.url();
        // Create a mock
        let mock = server
            .mock("GET", "/1978-mtg-loskort/")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(html_content.clone())
            .create();

        let stuff = get_all_card_pages(&url).await.unwrap();
        mock.assert();
        assert_eq!(stuff, endings())
    }

    #[test]
    fn test_card_parser() {
        init();
        let card_html = fs::read_to_string(
            "/workspaces/mtg-prz-rust/mtg-rust/src/alphaspel/alphaspel_cards_page.html",
        )
        .unwrap();

        let document = Html::parse_document(&card_html);
        // print!("{:?}", document);
        let selector = Selector::parse(".products.row div.product").unwrap();
        let products = document.select(&selector);

        let mut cards: Vec<VendorCard> = Vec::new();
        for product in products {
            match get_card_information(product) {
                Ok(card) => cards.push(card),
                Err(e) => {
                    log::error!("Error parsing card: {}", e);
                }
            }
        }
        let first_card = VendorCard {
            name: CardName::new("Loxodon Mystic".to_owned()).unwrap(),
            vendor: Vendor::Alphaspel,
            foil:false,
            image_url: "https://alphaspel.se/media/products/thumbs/e29e66d6-1d3b-4824-9ae4-a8607aec5648.250x250_q50_fill.png".to_owned(),
            extended_art: false,
            prerelease: false,
            showcase: false,
            set: SetName::new("10th Edition".to_owned()).unwrap(),
            price: 5,
            trade_in_price: 0,
            current_stock: 9,
            max_stock: 3,
        };

        assert_eq!(cards.len(), 51);
        assert_eq!(cards[0], first_card);
        assert_eq!(cards[49].extended_art, true);
        assert_eq!(cards.last().unwrap().prerelease, true);
        assert_eq!(cards.last().unwrap().foil, true);
        assert_eq!(
            cards.last().unwrap().name.almost_raw,
            "Whiskervale Forerunner"
        );
    }

    // #[tokio::test]
    // async fn test_fetch_and_parse() {
    //     init();

    //     let url = "https://alphaspel.se"; // server.url();
    //     let stuff = download_alpha_cards(&url).await.unwrap();
    //     print!("{}", stuff);
    // }

    fn endings() -> Vec<String> {
        return vec![
            "/2441-10th-edition/".to_string(),
            "/4130-30th-anniversary-celebration/".to_string(),
            "/2149-4th-edition/".to_string(),
            "/4555-4th-edition-black-bordered/".to_string(),
            "/2150-5th-edition/".to_string(),
            "/2556-6th-edition/".to_string(),
            "/2201-7th-edition/".to_string(),
            "/2571-8th-edition/".to_string(),
            "/2583-9th-edition/".to_string(),
            "/3612-adventures-in-the-forgotten-realms/".to_string(),
            "/2331-aether-revolt/".to_string(),
            "/2174-alara-reborn/".to_string(),
            "/2145-alliances/".to_string(),
            "/2895-alpha/".to_string(),
            "/2369-amonkhet/".to_string(),
            "/2586-amonkhet-invocations/".to_string(),
            "/4133-anthologies/".to_string(),
            "/2631-antiquities/".to_string(),
            "/2062-apocalypse/".to_string(),
            "/3335-arabian-nights/".to_string(),
            "/2559-archenemy/".to_string(),
            "/4914-assassins-creed/".to_string(),
            "/2165-avacyn-restored/".to_string(),
            "/2110-battle-for-zendikar/".to_string(),
            "/2654-battlebond/".to_string(),
            "/2649-beta/".to_string(),
            "/2552-betrayers-of-kamigawa/".to_string(),
            "/4919-bloomburrow/".to_string(),
            "/2011-born-of-the-gods/".to_string(),
            "/4853-breaking-news/".to_string(),
            "/2127-champions-of-kamigawa/".to_string(),
            "/2225-chronicles/".to_string(),
            "/2095-coldsnap/".to_string(),
            "/2301-commander/".to_string(),
            "/2250-commander-2013/".to_string(),
            "/2270-commander-2014/".to_string(),
            "/2271-commander-2015/".to_string(),
            "/2342-commander-2016/".to_string(),
            "/2518-commander-2017/".to_string(),
            "/2729-commander-2018/".to_string(),
            "/3101-commander-2019/".to_string(),
            "/3099-commander-2020/".to_string(),
            "/2429-commander-anthology/".to_string(),
            "/2661-commander-anthology-2018/".to_string(),
            "/4094-commander-collection-black/".to_string(),
            "/4448-commander-collection-green/".to_string(),
            "/3273-commander-legends/".to_string(),
            "/3974-commander-legends-battle-for-baldurs-gate/".to_string(),
            "/3981-commander-legends-battle-for-baldurs-gate-commander-decks/".to_string(),
            "/4594-commander-masters/".to_string(),
            "/3613-commander-adventures-in-the-forgotten-realms/".to_string(),
            "/4922-commander-bloomburrow/".to_string(),
            "/4033-commander-dominaria-united/".to_string(),
            "/3773-commander-innistrad-crimson-vow/".to_string(),
            "/3708-commander-innistrad-midnight-hunt/".to_string(),
            "/3888-commander-kamigawa-neon-dynasty/".to_string(),
            "/4525-commander-march-of-the-machine/".to_string(),
            "/4898-commander-modern-horizons-3/".to_string(),
            "/4712-commander-murders-at-karlov-manor/".to_string(),
            "/4851-commander-outlaws-of-thunder-junction/".to_string(),
            "/4458-commander-phyrexia-all-will-be-one/".to_string(),
            "/3967-commander-streets-of-new-capenna/".to_string(),
            "/3478-commander-strixhaven/".to_string(),
            "/4082-commander-the-brothers-war/".to_string(),
            "/4576-commander-the-lord-of-the-rings-tales-of-middle-earth/".to_string(),
            "/4663-commander-the-lost-caverns-of-ixalan/".to_string(),
            "/4601-commander-wilds-of-eldraine/".to_string(),
            "/2315-conflux/".to_string(),
            "/2025-conspiracy/".to_string(),
            "/2272-conspiracy-take-the-crown/".to_string(),
            "/2688-core-set-2019/".to_string(),
            "/2926-core-set-2020/".to_string(),
            "/3139-core-set-2021/".to_string(),
            "/2010-dark-ascension/".to_string(),
            "/2172-darksteel/".to_string(),
            "/2420-dissension/".to_string(),
            "/2625-dominaria/".to_string(),
            "/4131-dominaria-remastered/".to_string(),
            "/4024-dominaria-united/".to_string(),
            "/3142-double-masters/".to_string(),
            "/3992-double-masters-2022/".to_string(),
            "/2059-dragons-maze/".to_string(),
            "/2027-dragons-of-tarkir/".to_string(),
            "/2219-duel-decks-ajani-vs-nicol-bolas/".to_string(),
            "/2743-duel-decks-archenemy-nicol-bolas/".to_string(),
            "/2594-duel-decks-blessed-vs-cursed/".to_string(),
            "/2645-duel-decks-divine-vs-demonic/".to_string(),
            "/2585-duel-decks-elspeth-vs-tezzeret/".to_string(),
            "/2224-duel-decks-elspeth-vs-kiora/".to_string(),
            "/2780-duel-decks-elves-vs-goblins/".to_string(),
            "/2731-duel-decks-elves-vs-inventors/".to_string(),
            "/2193-duel-decks-garruk-vs-liliana/".to_string(),
            "/2218-duel-decks-heroes-vs-monsters/".to_string(),
            "/2216-duel-decks-izzet-vs-golgari/".to_string(),
            "/2587-duel-decks-jace-vs-chandra/".to_string(),
            "/2584-duel-decks-jace-vs-vraska/".to_string(),
            "/3349-duel-decks-knights-vs-dragons/".to_string(),
            "/3108-duel-decks-merfolk-vs-goblins/".to_string(),
            "/2566-duel-decks-mind-vs-might/".to_string(),
            "/2436-duel-decks-nissa-vs-ob-nixilis/".to_string(),
            "/2251-duel-decks-phyrexia-vs-the-coalition/".to_string(),
            "/2188-duel-decks-sorin-vs-tibalt/".to_string(),
            "/2228-duel-decks-speed-vs-cunning/".to_string(),
            "/2558-duel-decks-venser-vs-koth/".to_string(),
            "/2089-duel-decks-zendikar-vs-eldrazi/".to_string(),
            "/2564-duels-of-the-planeswalkers/".to_string(),
            "/2252-eldritch-moon/".to_string(),
            "/4600-enchanting-tales/".to_string(),
            "/2247-eternal-masters/".to_string(),
            "/2340-eventide/".to_string(),
            "/3350-exodus/".to_string(),
            "/3029-explorers-of-ixalan/".to_string(),
            "/2146-fallen-empires/".to_string(),
            "/1999-fate-reforged/".to_string(),
            "/2173-fifth-dawn/".to_string(),
            "/2575-foreign-black-bordered/".to_string(),
            "/4596-foreign-white-bordered/".to_string(),
            "/2194-from-the-vault-angels/".to_string(),
            "/2004-from-the-vault-annihilation/".to_string(),
            "/2789-from-the-vault-dragons/".to_string(),
            "/3724-from-the-vault-exiled/".to_string(),
            "/3348-from-the-vault-legends/".to_string(),
            "/2694-from-the-vault-lore/".to_string(),
            "/2213-from-the-vault-realms/".to_string(),
            "/3954-from-the-vault-relics/".to_string(),
            "/3032-from-the-vault-transform/".to_string(),
            "/2445-from-the-vault-twenty/".to_string(),
            "/2362-future-sight/".to_string(),
            "/2020-gatecrash/".to_string(),
            "/2217-gateway/".to_string(),
            "/2557-gpptfnm-promos/".to_string(),
            "/2765-guild-kits/".to_string(),
            "/2061-guildpact/".to_string(),
            "/2735-guilds-of-ravnica/".to_string(),
            "/4009-guilds-of-ravnica-mythic-edition/".to_string(),
            "/2226-homelands/".to_string(),
            "/2428-hour-of-devastation/".to_string(),
            "/2148-ice-age/".to_string(),
            "/2523-iconic-masters/".to_string(),
            "/3098-ikoria-lair-of-behemoths/".to_string(),
            "/2058-innistrad/".to_string(),
            "/3770-innistrad-crimson-vow/".to_string(),
            "/4494-innistrad-double-feature/".to_string(),
            "/3662-innistrad-midnight-hunt/".to_string(),
            "/2189-invasion/".to_string(),
            "/2517-ixalan/".to_string(),
            "/2006-journey-into-nyx/".to_string(),
            "/2653-judgment/".to_string(),
            "/3196-jumpstart/".to_string(),
            "/4093-jumpstart-2022/".to_string(),
            "/2293-kaladesh/".to_string(),
            "/2332-kaladesh-inventions/".to_string(),
            "/3339-kaldheim/".to_string(),
            "/3341-kaldheim-commander/".to_string(),
            "/3861-kamigawa-neon-dynasty/".to_string(),
            "/3960-kamigawa-neon-dynasty/".to_string(),
            "/1979-khans-of-tarkir/".to_string(),
            "/2234-legends/".to_string(),
            "/2553-legions/".to_string(),
            "/2009-lorwyn/".to_string(),
            "/2128-magic-2010/".to_string(),
            "/2222-magic-2011/".to_string(),
            "/2221-magic-2012/".to_string(),
            "/2094-magic-2013/".to_string(),
            "/2055-magic-2014/".to_string(),
            "/1980-magic-2015/".to_string(),
            "/4524-march-of-the-machine/".to_string(),
            "/4543-march-of-the-machine-the-aftermath/".to_string(),
            "/2608-masters-25/".to_string(),
            "/2008-mercadian-masques/".to_string(),
            "/2550-mirage/".to_string(),
            "/2336-mirrodin/".to_string(),
            "/2210-mirrodin-besieged/".to_string(),
            "/2014-modern-event-deck/".to_string(),
            "/2913-modern-horizons/".to_string(),
            "/3578-modern-horizons-2/".to_string(),
            "/4896-modern-horizons-3/".to_string(),
            "/2300-modern-masters-2013/".to_string(),
            "/2053-modern-masters-2015/".to_string(),
            "/2354-modern-masters-2017/".to_string(),
            "/2176-morningtide/".to_string(),
            "/4526-multiverse-legends/".to_string(),
            "/4708-murders-at-karlov-manor/".to_string(),
            "/3053-mystery-booster/".to_string(),
            "/3985-mystery-booster-playtest-cards/".to_string(),
            "/2227-nemesis/".to_string(),
            "/2211-new-phyrexia/".to_string(),
            "/2162-oath-of-the-gatewatch/".to_string(),
            "/2581-odyssey/".to_string(),
            "/2007-onslaught/".to_string(),
            "/2075-origins/".to_string(),
            "/4852-outlaws-of-thunder-junction/".to_string(),
            "/4456-phyrexia-all-will-be-one/".to_string(),
            "/2187-planar-chaos/".to_string(),
            "/2560-planechase/".to_string(),
            "/3028-planechase-2012/".to_string(),
            "/2658-planechase-anthology/".to_string(),
            "/2565-planeshift/".to_string(),
            "/2337-portal/".to_string(),
            "/2632-portal-second-age/".to_string(),
            "/4826-portal-three-kingdoms/".to_string(),
            "/2633-premium-deck-fire-and-lightning/".to_string(),
            "/2790-premium-deck-graveborn/".to_string(),
            "/2696-premium-deck-slivers/".to_string(),
            "/2555-prophecy/".to_string(),
            "/2130-ravnica/".to_string(),
            "/2791-ravnica-allegiance/".to_string(),
            "/2812-ravnica-allegiance-guild-kits/".to_string(),
            "/4694-ravnica-remasterd/".to_string(),
            "/2056-return-to-ravnica/".to_string(),
            "/2147-revised-3rd/".to_string(),
            "/2126-rise-of-the-eldrazi/".to_string(),
            "/2548-rivals-of-ixalan/".to_string(),
            "/2554-saviors-of-kamigawa/".to_string(),
            "/2175-scars-of-mirrodin/".to_string(),
            "/2582-scourge/".to_string(),
            "/3860-secret-lair-drop-series/".to_string(),
            "/4115-secret-lair-drop-series-30th-anniversary-countdown-kit/".to_string(),
            "/2341-shadowmoor/".to_string(),
            "/2196-shadows-over-innistrad/".to_string(),
            "/2347-shards-of-alara/".to_string(),
            "/4134-signature-spellbook-jace/".to_string(),
            "/4671-special-guests/".to_string(),
            "/4532-starter-commander-decks/".to_string(),
            "/3961-streets-of-new-capenna/".to_string(),
            "/3482-strixhaven-school-of-mages/".to_string(),
            "/2316-stronghold/".to_string(),
            "/2223-tempest/".to_string(),
            "/4854-the-big-score/".to_string(),
            "/4078-the-brothers-war/".to_string(),
            "/2574-the-dark/".to_string(),
            "/3354-the-list/".to_string(),
            "/4464-the-list-secret-lair/".to_string(),
            "/4575-the-lord-of-the-rings-tales-of-middle-earth/".to_string(),
            "/4662-the-lost-caverns-of-ixalan/".to_string(),
            "/2012-theros/".to_string(),
            "/3023-theros-beyond-death/".to_string(),
            "/2973-throne-of-eldraine/".to_string(),
            "/2215-time-spiral/".to_string(),
            "/3379-time-spiral-remastered/".to_string(),
            "/2561-timeshifted/".to_string(),
            "/2563-torment/".to_string(),
            "/2779-ultimate-masters/".to_string(),
            "/4062-unfinity/".to_string(),
            "/2549-unglued/".to_string(),
            "/2177-unhinged/".to_string(),
            "/4646-universes-beyond-doctor-who/".to_string(),
            "/4836-universes-beyond-fallout/".to_string(),
            "/4670-universes-beyond-jurassic-world-collection/".to_string(),
            "/4080-universes-beyond-transformers/".to_string(),
            "/4063-universes-beyond-warhammer-40000/".to_string(),
            "/4607-universes-within/".to_string(),
            "/2320-unlimited/".to_string(),
            "/3984-unsanctioned/".to_string(),
            "/2773-unstable/".to_string(),
            "/2317-urzas-destiny/".to_string(),
            "/2318-urzas-legacy/".to_string(),
            "/2060-urzas-saga/".to_string(),
            "/2212-visions/".to_string(),
            "/2892-war-of-the-spark/".to_string(),
            "/2748-weatherlight/".to_string(),
            "/4599-wilds-of-eldraine/".to_string(),
            "/4648-world-cup-decks/".to_string(),
            "/2335-worldwake/".to_string(),
            "/2057-zendikar/".to_string(),
            "/2181-zendikar-expeditions/".to_string(),
            "/3151-zendikar-rising/".to_string(),
            "/3156-zendikar-rising-commander/".to_string(),
        ];
    }
}
