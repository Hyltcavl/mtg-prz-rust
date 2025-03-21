# Magic card scraper

This repo contains code to scan some stores, compare the cards to mcm trend and to compare cards from delver lense with cards in store for tradein.

The pipeline runs daily and updates the index.html page witch cards that have "nice price".

## How to run

The defaults of the config is to run all screapers and use the MTG stocks compare api.

Basic run commands:
```bash

RUST_LOG=info DL=1 AS=1 SF=1 EXTERNAL_PRICE_CHECK=1 DELVER_LENSE_PATH=./../Draftshaft_2025_Mar_10_17-33.csv NICE_PRICE_DIFF=0 cargo run > output.log 2>&1

```

Run command for specific test:
```bash

RUST_LOG=info cargo test --package mtg-rust --bin mtg-rust -- delver_lense::delver_lense_card::tests2::test_fetch_card --exact --show-output > output.log 2>&1
```
## Other
How to add dependency:

to add dependency:
```bash

cargo add <dependency name>
```

```bash

cargo add <dependency name> --feature <feature name>
```


## On the TODO list

- Redesign alphaspel card parser to save more of the card raw name
- Update alphaspel_scraper not to error log all cards it doesnt use, and improve its logs
- Get Manatorsk cards

## Known issues:
- DL card names have varying qualitites, and double faced cards usually only have the name of one of the faces while the scryfall cards usually have both \(also some variation there\).
- Collector numbers on DL might not match the collector number on the scryfall card, but then it defaults to the name and set name that might be too including some times. 
