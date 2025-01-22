use serde::{Serialize, Deserialize};

use crate::blockchain::deckchain::DeckChain;

use super::series::TradingCardSeries;
use super::card::{self, TradingCard};

use rand::{rngs::StdRng, Rng, SeedableRng};
use rand::seq::SliceRandom;

#[derive(Clone, Serialize, Deserialize)]
pub struct TradingCardSeriesRelease {
    pub id: String,
    pub series: TradingCardSeries,
    pub released_cards: Vec<TradingCard>,
    pub shuffle_hash: String,
}

impl TradingCardSeriesRelease {
    pub async fn from_deckchain(deckchain: DeckChain, series_id: String) -> Self {
        let series = TradingCardSeries::from_deckchain(deckchain, series_id.clone()).await;

        TradingCardSeriesRelease {
            id: series_id,
            series: series.unwrap(),
            released_cards: Vec::new(),
            shuffle_hash: String::new(),
        }
    }

    pub async fn save_to_deckchain(&self, deckchain: DeckChain) {
        deckchain.release_series(id, series, shuffle_hash).await;
    }

    pub async fn new_from_series(series: TradingCardSeries, private_salt: [u8; 16]) -> Self {
        let mut release = TradingCardSeriesRelease {
            id: series.id.clone(),
            series,
            released_cards: Vec::new(),
            shuffle_hash: String::new(),
        };
        let shuffle_hash = TradingCardSeriesRelease::generate_shuffle_hash();
        release.shuffle_hash = hex::encode(shuffle_hash);
        release.build_cards(private_salt).await;
        release
    }

    pub async fn new_from_series_to_deckchain(series: TradingCardSeries, deckchain: DeckChain, private_salt: [u8; 16]) {
        let release = TradingCardSeriesRelease::new_from_series(series, private_salt).await;
        release.save_to_deckchain(deckchain).await;
    }

    pub async fn build_cards(&mut self, private_salt: [u8; 16]) {
        let mut card_deck: Vec<TradingCard> = Vec::new();
        let card_configs = self.series.get_card_configs().await;
        let mint_list: Vec<Vec<String>> = self.series.get_mint_list().await;
        let mint_list_length = mint_list.len() as u32;

        for card in card_configs.clone() {
            let mut serial = 1;
            for properties in mint_list.clone() {
                let card = TradingCard::from_card_config(
                    &card,
                    properties.clone(),
                    self.id.clone(),
                    TradingCardSeriesRelease::format_serial(serial, mint_list_length)
                );
                card_deck.push(card);
                serial += 1;
            }
        }

        let mut seeded_rng = TradingCardSeriesRelease::get_seeded_rng(
            self.get_shuffle_hash(),
            private_salt
        );

        card_deck.shuffle(&mut seeded_rng);
        self.released_cards = card_deck;
    }

    fn format_serial(serial: u32, mint_count: u32) -> String {
        let mint_count_length = mint_count.to_string().len();
        format!("{:0width$}", serial, width=mint_count_length)
    }

    fn get_shuffle_hash(&self) -> [u8; 16] {
        let mut shuffle_hash = [0u8; 16];
        shuffle_hash.copy_from_slice(&hex::decode(self.shuffle_hash.clone()).unwrap());
        shuffle_hash
    }

    fn get_seeded_rng(shuffle_hash: [u8; 16], private_salt: [u8; 16]) -> StdRng {
        let mut rng_seed_bytes: [u8; 32] = [0; 32];
        rng_seed_bytes[..16].copy_from_slice(&shuffle_hash);
        rng_seed_bytes[16..].copy_from_slice(&private_salt[..16]);
        StdRng::from_seed(rng_seed_bytes)
    }

    fn generate_shuffle_hash() -> [u8; 16] {
        let mut rng = rand::thread_rng();
        let mut public_key = [0u8; 16];
        rng.fill(&mut public_key);
        public_key
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use crate::card::series::tests::test_series_data;

    #[tokio::test]
    async fn test_series_release() {
        let series_release = get_testing_release().await;
        assert!(series_release.released_cards.len() > 0);
    }

    #[tokio::test]
    async fn test_num_minted_cards() {
        let series_release = get_testing_release().await;
        let desired_num_cards = series_release.series.get_mint_total().await;
        let minted_cards = series_release.released_cards.len() as u32;

        assert_eq!(desired_num_cards, minted_cards, "Expected minted {} cards, got {}", desired_num_cards, minted_cards);
    }

    #[tokio::test]
    async fn test_build_cards() {
        let series_release = get_testing_release().await;
        let series_data = test_series_data().await;
        let mint_list = series_data.get_mint_list().await;
        let num_distinct_cards = series_release.series.get_card_configs().await.len() as u32;
        let mut released_cards_prop_counter = HashMap::new();

        for card in series_release.released_cards.iter() {
            for prop in card.properties.iter() {
                *released_cards_prop_counter.entry(prop).or_insert(0) += 1;
            }
        }

        // Generate the same type of list for the mint list.
        let mut mint_prop_counter = HashMap::new();
        for props in mint_list.iter() {
            for prop in props.iter() {
                *mint_prop_counter.entry(prop).or_insert(0) += 1;
            }
        }

        for (mint_prop, mint_count) in mint_prop_counter.iter() {
            let prop_count = released_cards_prop_counter.get(*mint_prop).unwrap_or(&0);
            let expected_mint_amount = mint_count * num_distinct_cards as u32;
            assert_eq!(*prop_count, expected_mint_amount, "Property {} was minted {} times, expected {}", mint_prop, prop_count, expected_mint_amount);
        }
    }

    #[tokio::test]
    async fn test_repeatable_shuffle_seed() {
        let series_release = get_testing_release().await;
        let series_release_copy = series_release.clone();

        // Since these are shuffled with a seed, they should be the same.
        for (card, card_copy) in series_release.released_cards.iter().zip(series_release_copy.released_cards.iter()) {
            assert_eq!(card, card_copy);
        }
    }

    async fn get_testing_release() -> TradingCardSeriesRelease {
        let series = test_series_data().await;
        let mut release = TradingCardSeriesRelease {
            id: series.id.clone(),
            series,
            released_cards: Vec::new(),
            shuffle_hash: String::new(),
        };
        let (shuffle_hash, private_salt) = get_testing_hash_salt();
        release.shuffle_hash = hex::encode(shuffle_hash);
        release.build_cards(private_salt).await;
        release
    }

    fn get_testing_hash_salt() -> ([u8; 16], [u8; 16]) {
        let mut private_salt = "76f38e455a57ed4003bfd1a1c83cc9e5";
        let mut shuffle_hash = "935a5191ff1e7dbd10df7f0957da72ae";
        
        let mut private_salt_bytes: [u8; 16] = [0; 16];
        let mut shuffle_hash_bytes: [u8; 16] = [0; 16];

        private_salt_bytes.copy_from_slice(&hex::decode(private_salt).unwrap());
        shuffle_hash_bytes.copy_from_slice(&hex::decode(shuffle_hash).unwrap());

        (shuffle_hash_bytes, private_salt_bytes)
    }

}