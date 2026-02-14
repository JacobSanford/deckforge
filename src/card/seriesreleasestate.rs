use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::blockchain::deckchain::DeckChain;
use crate::error::Result;

use super::card::TradingCard;
use super::series::TradingCardSeries;

#[derive(Clone, Serialize, Deserialize)]
pub struct TradingCardSeriesReleaseState {
    pub id: String,
    pub series: TradingCardSeries,
    pub released_cards: Vec<TradingCard>,
    pub shuffle_hash: String,
}

impl TradingCardSeriesReleaseState {
    pub fn from_deckchain(deckchain: &DeckChain, series_id: String) -> Result<Self> {
        let series = TradingCardSeries::from_deckchain(deckchain, series_id.clone())?;

        Ok(TradingCardSeriesReleaseState {
            id: series_id,
            series,
            released_cards: Vec::new(),
            shuffle_hash: String::new(),
        })
    }

    pub fn new_from_series(series: TradingCardSeries, private_salt: [u8; 16]) -> Self {
        let mut release = TradingCardSeriesReleaseState {
            id: series.id.clone(),
            series,
            released_cards: Vec::new(),
            shuffle_hash: String::new(),
        };
        let shuffle_hash = TradingCardSeriesReleaseState::generate_shuffle_hash();
        release.shuffle_hash = hex::encode(shuffle_hash);
        release.build_cards(private_salt);
        release
    }

    pub fn build_cards(&mut self, private_salt: [u8; 16]) {
        let card_configs = self.series.get_card_configs();
        let mint_list = self.series.get_mint_list();
        let mint_list_length = mint_list.len() as u32;

        let mut card_deck: Vec<TradingCard> = Vec::with_capacity(
            card_configs.len() * mint_list.len(),
        );

        for card_config in card_configs {
            for (idx, properties) in mint_list.iter().enumerate() {
                let card = TradingCard::from_card_config(
                    card_config,
                    properties.clone(),
                    self.id.clone(),
                    TradingCardSeriesReleaseState::format_serial(
                        (idx + 1) as u32,
                        mint_list_length,
                    ),
                );
                card_deck.push(card);
            }
        }

        let mut seeded_rng = TradingCardSeriesReleaseState::get_seeded_rng(
            self.get_shuffle_hash(),
            private_salt,
        );

        card_deck.shuffle(&mut seeded_rng);
        self.released_cards = card_deck;
    }

    fn format_serial(serial: u32, mint_count: u32) -> String {
        let mint_count_length = mint_count.to_string().len();
        format!("{:0width$}", serial, width = mint_count_length)
    }

    fn get_shuffle_hash(&self) -> [u8; 16] {
        let mut shuffle_hash = [0u8; 16];
        let decoded = hex::decode(&self.shuffle_hash).unwrap_or_else(|_| vec![0u8; 16]);
        let len = decoded.len().min(16);
        shuffle_hash[..len].copy_from_slice(&decoded[..len]);
        shuffle_hash
    }

    fn get_seeded_rng(shuffle_hash: [u8; 16], private_salt: [u8; 16]) -> StdRng {
        let mut rng_seed_bytes: [u8; 32] = [0; 32];
        rng_seed_bytes[..16].copy_from_slice(&shuffle_hash);
        rng_seed_bytes[16..].copy_from_slice(&private_salt);
        StdRng::from_seed(rng_seed_bytes)
    }

    fn generate_shuffle_hash() -> [u8; 16] {
        let mut rng = rand::thread_rng();
        let mut hash = [0u8; 16];
        rng.fill(&mut hash);
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use crate::card::series::tests::test_series_data;

    #[test]
    fn test_series_release() {
        let series_release = get_testing_release();
        assert!(!series_release.released_cards.is_empty());
    }

    #[test]
    fn test_num_minted_cards() {
        let series_release = get_testing_release();
        let desired_num_cards = series_release.series.get_mint_total();
        let minted_cards = series_release.released_cards.len() as u32;

        assert_eq!(
            desired_num_cards, minted_cards,
            "Expected minted {} cards, got {}",
            desired_num_cards, minted_cards
        );
    }

    #[test]
    fn test_build_cards() {
        let series_release = get_testing_release();
        let series_data = test_series_data();
        let mint_list = series_data.get_mint_list();
        let num_distinct_cards = series_release.series.get_card_configs().len() as u32;
        let mut released_cards_prop_counter: HashMap<&str, u32> = HashMap::new();

        for card in series_release.released_cards.iter() {
            for prop in card.properties.iter() {
                *released_cards_prop_counter.entry(prop).or_insert(0) += 1;
            }
        }

        let mut mint_prop_counter: HashMap<&str, u32> = HashMap::new();
        for props in mint_list.iter() {
            for prop in props.iter() {
                *mint_prop_counter.entry(prop).or_insert(0) += 1;
            }
        }

        for (mint_prop, mint_count) in mint_prop_counter.iter() {
            let prop_count = released_cards_prop_counter.get(mint_prop).unwrap_or(&0);
            let expected_mint_amount = mint_count * num_distinct_cards;
            assert_eq!(
                *prop_count, expected_mint_amount,
                "Property {} was minted {} times, expected {}",
                mint_prop, prop_count, expected_mint_amount
            );
        }
    }

    #[test]
    fn test_repeatable_shuffle_seed() {
        let series_release = get_testing_release();
        let series_release_copy = get_testing_release();

        for (card, card_copy) in series_release
            .released_cards
            .iter()
            .zip(series_release_copy.released_cards.iter())
        {
            assert_eq!(card, card_copy);
        }
    }

    fn get_testing_release() -> TradingCardSeriesReleaseState {
        let series = test_series_data();
        let mut release = TradingCardSeriesReleaseState {
            id: series.id.clone(),
            series,
            released_cards: Vec::new(),
            shuffle_hash: String::new(),
        };
        let (shuffle_hash, private_salt) = get_testing_hash_salt();
        release.shuffle_hash = hex::encode(shuffle_hash);
        release.build_cards(private_salt);
        release
    }

    fn get_testing_hash_salt() -> ([u8; 16], [u8; 16]) {
        let private_salt = "76f38e455a57ed4003bfd1a1c83cc9e5";
        let shuffle_hash = "935a5191ff1e7dbd10df7f0957da72ae";

        let mut private_salt_bytes: [u8; 16] = [0; 16];
        let mut shuffle_hash_bytes: [u8; 16] = [0; 16];

        private_salt_bytes.copy_from_slice(&hex::decode(private_salt).unwrap());
        shuffle_hash_bytes.copy_from_slice(&hex::decode(shuffle_hash).unwrap());

        (shuffle_hash_bytes, private_salt_bytes)
    }
}
