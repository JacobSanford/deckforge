use std::fs::File;
use std::io::BufReader;
use std::io;
use std::error::Error;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::blockchain::deckchain::DeckChain;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TradingCardSeries {
    pub id: String,
    name: String,
    description: String,
    config: Config,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    distribution: Distribution,
    cards: Vec<CardConfig>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Distribution {
    rarity: std::collections::HashMap<String, Rarity>,
    mint: Mint,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Rarity {
    name: String,
    items: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Mint {
    total: u32,
    special: std::collections::HashMap<String, Special>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Special {
    name: String,
    items: u32,
    numbered_sets: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CardConfig {
    pub number: u32,
    pub title: String,
    pub description: String,
    pub rarity: u32,
    #[serde(rename = "type")]
    pub card_type: String,
}

impl TradingCardSeries {

    pub const MAXIMUM_RARITY_VALUE: u32 = u32::MAX;
    pub const ERROR_NO_ID: &'static str = "Series JSON does not contain an 'id' field";
    pub const ERROR_NO_NAME: &'static str = "Series JSON does not contain a 'name' field";
    pub const ERROR_NO_DESCRIPTION: &'static str = "Series JSON does not contain a 'description' field";
    pub const ERROR_NO_CARDS: &'static str = "Series JSON does not contain a 'cards' field";
    pub const ERROR_TOTAL_CARDS_MISMATCH: &'static str = "Total cards in series does not match the rarity distribution";
    pub const ERROR_CARD_RARITIES_MISMATCH: &'static str = "Card rarities do not match the rarity distribution";
    pub const ERROR_RARITY_ORDER: &'static str = "Rarity order is not sensible";
    pub const ERROR_RARITY_RATIO: &'static str = "Rarity ratio exceeds 50% between tiers";

    pub async fn from_file(series_file: &str) -> Result<Self, io::Error> {
        let file = File::open(series_file).unwrap();
        let reader = BufReader::new(file);
        let series: TradingCardSeries = serde_json::from_reader(reader).unwrap();
        let validator = series.validate_series().await;
        if let Err(e) = validator {
            return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
        }
        Ok(series)
    }

    pub async fn from_deckchain(deckchain: DeckChain, series_id: String) -> Result<Self, io::Error> {
        let series_value = deckchain.card_series_release(series_id).await.unwrap();
        let series: TradingCardSeries = serde_json::from_value(series_value).unwrap();
        let validator = series.validate_series().await;
        if let Err(e) = validator {
            return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
        }
        Ok(series)
    }

    pub async fn get_card_configs(&self) -> Vec<CardConfig> {
        self.config.cards.clone()
    }

    pub async fn get_mint_each(&self) -> u32 {
        self.config.distribution.mint.total
    }

    pub async fn get_mint_total(&self) -> u32 {
        let num_ea = self.get_mint_each().await;
        let desired_num_cards = num_ea * self.get_card_configs().await.len() as u32;
        desired_num_cards
    }

    pub async fn get_specials(&self) -> std::collections::HashMap<String, Special> {
        self.config.distribution.mint.special.clone()
    }

    pub async fn get_mint_list(&self) -> Vec<Vec<String>> {
        let mint_ea = self.get_mint_each().await;

        let mut mint_list = Vec::new();

        // Sort the specials by key to ensure consistent order
        let mut sorted_specials: Vec<_> = self.config.distribution.mint.special.iter().collect();
        sorted_specials.sort_by(|a, b| a.0.cmp(b.0));

        for (_key, special) in sorted_specials {
            for _i in 0..special.items {
                mint_list.push(vec![special.name.clone()]);
            }
        }

        let specials_minted = mint_list.len() as u32;
        for _j in 0..mint_ea - specials_minted {
            mint_list.push(vec!["".to_string()]);
        }
        mint_list
    }

    pub async fn validate_series(&self) -> Result<(), Box<dyn Error>> {
        let series_values_result = self.validate_series_values().await;
        if series_values_result.is_err() {
            return series_values_result;
        }

        Ok(())
    }

    pub async fn validate_series_values(&self) -> Result<(), Box<dyn Error>> {
        if self.id.is_empty() {
            return Err(Self::ERROR_NO_ID.into());
        }

        if self.name.is_empty() {
            return Err(Self::ERROR_NO_NAME.into());
        }

        if self.description.is_empty() {
            return Err(Self::ERROR_NO_DESCRIPTION.into());
        }

        if self.config.cards.is_empty() {
            return Err(Self::ERROR_NO_CARDS.into());
        }

        if !self.rarity_order_is_sensible().await {
            return Err(Self::ERROR_RARITY_ORDER.into());
        }

        if !self.rarity_ratios_are_sensible().await {
            return Err(Self::ERROR_RARITY_RATIO.into());
        }

        if !self.total_cards_match().await {
            return Err(Self::ERROR_TOTAL_CARDS_MISMATCH.into());
        }

        if !self.card_rarities_match_card_configs().await {
            return Err(Self::ERROR_CARD_RARITIES_MISMATCH.into());
        }

        if !self.mint_count_is_reasonable().await {
            return Err("You should mint at least one of each card".into());
        }

        if !self.mint_specials_counts_are_sensible().await {
            return Err("Mint special cards shouldn't exceed more than 1/2.5 of printed cards".into());
        }

        if !self.mint_special_order_is_sensible().await {
            return Err("Mint Special order is not sensible".into());
        }
        Ok(())
    }

    pub async fn mint_special_order_is_sensible(&self) -> bool {
        let mint_specials = self.get_mint_specials_as_sorted_vec().await;
        let mut last_special = (Self::MAXIMUM_RARITY_VALUE / 2) - 1;
        for (_key, special) in mint_specials.iter() {
            if *special > (last_special / 2) {
                return false;
            }

            last_special = *special;
        }
        true
    }

    // Total specials should not exceed 1/2.5 of the mint_ea.
    pub async fn mint_specials_counts_are_sensible(&self) -> bool {
        let mint_specials = self.get_mint_specials_specs().await;
        let mint_each = self.get_mint_each().await;
        let mut total_specials = 0;
        for (_key, special) in mint_specials.iter() {
            total_specials += special;
        }
        total_specials < (mint_each as f32 / 2.5) as u32
    }

    pub async fn mint_count_is_reasonable(&self) -> bool {
        let mint_each = self.get_mint_each().await;
        mint_each > 0
    }

    pub async fn rarity_ratios_are_sensible(&self) -> bool {
        let rarity_value_counts = self.rarity_spec_as_sorted_vec().await;
        let mut last_rarity = (Self::MAXIMUM_RARITY_VALUE / 2) - 1;
        for (_key, rarity) in rarity_value_counts.iter() {
            if *rarity > (last_rarity / 2) {
                return false;
            }

            last_rarity = *rarity;
        }
        true
    }

    pub async fn rarity_order_is_sensible(&self) -> bool {
        let rarity_value_counts = self.rarity_spec_as_sorted_vec().await;
        let mut last_rarity = Self::MAXIMUM_RARITY_VALUE;
        for (_key, rarity) in rarity_value_counts.iter() {
            if *rarity > last_rarity {
                return false;
            }
            last_rarity = *rarity;
        }
        true
    }

    pub async fn get_mint_specials_specs(&self) -> HashMap<String, u32> {
        let mut mint_specials = HashMap::new();
        for (key, special) in self.config.distribution.mint.special.iter() {
            mint_specials.insert(key.to_string(), special.items);
        }
        mint_specials
    }

    pub async fn get_mint_specials_as_sorted_vec(&self) -> Vec<(String, u32)> {
        let mut mint_specials = self.get_mint_specials_specs().await;
        let mut mint_specials_vec: Vec<(String, u32)> = mint_specials.drain().collect();
        mint_specials_vec.sort_by(|a, b| a.0.cmp(&b.0));
        mint_specials_vec
    }

    pub async fn get_rarity_specs(&self) -> HashMap<u32, u32> {
        let mut rarity_value_counts = HashMap::new();
        for (key, rarity) in self.config.distribution.rarity.iter() {
            let key: u32 = key.parse().unwrap();
            rarity_value_counts.insert(key, rarity.items);
        }
        rarity_value_counts
    }

    pub async fn rarity_spec_as_sorted_vec(&self) -> Vec<(u32, u32)> {
        let mut rarity_value_counts = self.get_rarity_specs().await;
        let mut rarity_vec: Vec<(u32, u32)> = rarity_value_counts.drain().collect();
        rarity_vec.sort_by(|a, b| a.0.cmp(&b.0));
        rarity_vec
    }

    pub async fn total_cards_match(&self) -> bool {
        let mut total_card_configs = 0;
        for _card_config in self.config.cards.iter() {
            total_card_configs += 1;
        }

        let mut total_rarity_cards = 0;
        for rarity in self.config.distribution.rarity.values() {
            total_rarity_cards += rarity.items;
        }

        total_card_configs == total_rarity_cards
    }

    pub async fn card_rarities_match_card_configs(&self) -> bool {
        let mut card_rarity_counts = HashMap::new();
        for card_config in self.config.cards.iter() {
            *card_rarity_counts.entry(card_config.rarity).or_insert(0) += 1;
        }

        let rarity_value_counts = self.get_rarity_specs().await;

        card_rarity_counts == rarity_value_counts
    }

}

#[cfg(test)]
pub mod tests {
    use super::*;

    use rand::seq::SliceRandom;
    use serde_json::Value;

    use crate::card::series::TradingCardSeries;

    #[tokio::test]
    async fn test_series_ok() {
        let series = test_series_data().await;
        let series_values_result = series.validate_series_values().await;
        assert!(series_values_result.is_ok());
    }

    #[tokio::test]
    async fn test_series_no_id() {
        let mut series = test_series_data().await;
        series.id = "".to_string();
        let series_values_result = series.validate_series_values().await;
        assert!(series_values_result.is_err());
        assert_eq!(series_values_result.err().unwrap().to_string(), TradingCardSeries::ERROR_NO_ID);
    }

    #[tokio::test]
    async fn test_series_data_from_file() {
        let series = test_series_data().await;
        assert_eq!(series.id, "LEGACYDECK-1");
    }

    #[tokio::test]
    async fn test_series_no_description() {
        let mut series = test_series_data().await;
        series.description = "".to_string();
        let series_values_result = series.validate_series_values().await;
        assert!(series_values_result.is_err());
        assert_eq!(series_values_result.err().unwrap().to_string(), TradingCardSeries::ERROR_NO_DESCRIPTION);
    }

    #[tokio::test]
    async fn test_series_no_name() {
        let mut series = test_series_data().await;
        series.name = "".to_string();
        let series_values_result = series.validate_series_values().await;
        assert!(series_values_result.is_err());
        assert_eq!(series_values_result.err().unwrap().to_string(), TradingCardSeries::ERROR_NO_NAME);
    }

    #[tokio::test]
    async fn test_series_no_cards() {
        let mut series = test_series_data().await;
        series.config.cards = Vec::new();
        let series_values_result = series.validate_series_values().await;
        assert!(series_values_result.is_err());
        assert_eq!(series_values_result.err().unwrap().to_string(), TradingCardSeries::ERROR_NO_CARDS);
    }

    // Below here we test specific funcs.
    #[tokio::test]
    async fn test_series_mismatch_total_cards() {
        let mut series = test_series_data().await;
        let number_to_remove = get_random_number(&series).await;
        series.config.cards.retain(|x| x.number != number_to_remove);

        let series_values_result = series.total_cards_match().await;
        assert_eq!(series_values_result, false);
    }

    #[tokio::test]
    async fn test_series_mismatch_card_rarities() {
        let mut series = test_series_data().await;

        // Pick a random rarity.
        let rarities = get_card_rarities(&series).await;
        let rarity_to_change = rarities.choose(&mut rand::thread_rng()).unwrap();
        let rarity_to_set = rarities.iter().find(|&&x| x != *rarity_to_change).unwrap();

        // Change it.
        for card in series.config.cards.iter_mut() {
            if card.rarity == *rarity_to_change {
                card.rarity = *rarity_to_set;
                break;
            }
        }

        let series_values_result = series.card_rarities_match_card_configs().await;
        assert_eq!(series_values_result, false);
    }

    // For rarity spec to be sensible, its item count must be in descending order according to key.
    #[tokio::test]
    async fn test_rarity_order_is_sensible() {
        let mut series = test_series_data().await;

        // Get a sorted list. Hashmap is not sorted.
        let mut rarity_vec = series.rarity_spec_as_sorted_vec().await;
        let rarity_count = rarity_vec.len() as u32;
    
        print!("{:?}", rarity_vec);

        // Reverse key order.
        for (key, _rarity) in rarity_vec.iter_mut() {
            *key = rarity_count - *key;
        }

        // Iterate and push these values to the series.
        for (key, rarity) in rarity_vec.iter() {
            series.config.distribution.rarity.insert(key.to_string(), Rarity {
                name: key.to_string(),
                items: *rarity,
            });
        }
    
        let series_values_result = series.rarity_order_is_sensible().await;
        assert_eq!(series_values_result, false);
    }

    // For rarity spec to be sensible, the ratio between each rarity tier must not exceed 50%.
    #[tokio::test]
    async fn test_rarity_ratios_are_sensible() {
        let mut series = test_series_data().await;
        let mut last_rarity = 256;

        // Get a sorted list. Hashmap is not sorted.
        let mut rarity_vec = series.rarity_spec_as_sorted_vec().await;

        // Set values to slightly more than 50% of the previous value.
        for (_key, rarity) in rarity_vec.iter_mut() {
            *rarity = (last_rarity / 2) + 1;
            last_rarity = *rarity;
        }

        // Iterate and push these values to the series.
        for (key, rarity) in rarity_vec.iter() {
            series.config.distribution.rarity.insert(key.to_string(), Rarity {
                name: key.to_string(),
                items: *rarity,
            });
        }
        print!("{:?}", series.config.distribution.rarity);

        let series_values_result = series.rarity_ratios_are_sensible().await;
        assert_eq!(series_values_result, false);
    }

    #[tokio::test]
    async fn test_mint_ea_is_sensible() {
        let mut series = test_series_data().await;
        series.config.distribution.mint.total = 0;
        let series_values_result = series.mint_count_is_reasonable().await;
        assert_eq!(series_values_result, false);
    }

    #[tokio::test]
    async fn test_mint_specials_order_is_sensible() {
        let mut series = test_series_data().await;
        let mint_specials = series.get_mint_specials_as_sorted_vec().await;
        let mut last_special = (TradingCardSeries::MAXIMUM_RARITY_VALUE / 2) - 1;
        for (_key, special) in mint_specials.iter() {
            if *special > (last_special / 2) {
                let series_values_result = series.mint_special_order_is_sensible().await;
                assert_eq!(series_values_result, false);
                return;
            }

            last_special = *special;
        }
        let series_values_result = series.mint_special_order_is_sensible().await;
        assert_eq!(series_values_result, true);
    }

    #[tokio::test]
    async fn test_mint_specials_counts_are_sensible() {
        let mut series = test_series_data().await;
        let mint_each = series.get_mint_each().await;

        if let Some((_key, special)) = series.config.distribution.mint.special.iter_mut().next() {
            special.items = mint_each / 2;
        }

        let series_values_result = series.mint_specials_counts_are_sensible().await;
        assert_eq!(series_values_result, false);
    }

    #[tokio::test]
    async fn test_mint_list() {
        let series = test_series_data().await;
        let mint_list = series.get_mint_list().await;
        let mint_ea = series.get_mint_each().await;

        let special_list = series.config.distribution.mint.special.clone();

        assert_eq!(mint_list.len() as u32, mint_ea);

        let mut prop_counter = HashMap::new();
        for props in mint_list.iter() {
            for prop in props.iter() {
                *prop_counter.entry(prop).or_insert(0) += 1;
            }
        }

        for special_item in special_list.iter() {
            let special = special_item.1;
            let special_name = special.name.clone();
            let special_items = special.items;

            let special_count = prop_counter.get(&special_name).unwrap_or(&0);
            assert_eq!(special_items, *special_count, "Special {} was minted {} times, expected {}", special_name, special_count, *special_count);
        }
    }

    #[tokio::test]
    async fn test_get_mint_each() {
        let series = test_series_data().await;
        let mint_each = series.get_mint_each().await;
        assert_eq!(mint_each, 242 as u32);
    }

    #[tokio::test]
    async fn test_get_mint_total() {
        let series = test_series_data().await;
        let mint_total = series.get_mint_total().await;
        let mint_each = series.get_mint_each().await;
        let card_configs = series.get_card_configs().await;
        let desired_num_cards = mint_each * card_configs.len() as u32;
        assert_eq!(mint_total, desired_num_cards);
    }

    /// Helper function to get the different card rarities from the series.
    async fn get_card_rarities(series: &TradingCardSeries) -> Vec<u32> {
        let mut rarities = Vec::new();
        for card in series.config.cards.iter() {
            rarities.push(card.rarity);
        }
        rarities
    }

    /// Helper function to get a random card number from the series.
    async fn get_random_number(series: &TradingCardSeries) -> u32 {
        let card = series.config.cards.choose(&mut rand::thread_rng());
        card.unwrap().number
    }

    pub fn test_series_json() -> Value {
        let series_file = "test/series.json".to_string();
        let file = File::open(series_file).unwrap();
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader).unwrap();
        data
    }

    /// Helper function to reads the test series data from the test file.
    pub async fn test_series_data() -> TradingCardSeries {
        let series_file = "test/series.json".to_string();
        let series_data = TradingCardSeries::from_file(series_file.as_str()).await;
        series_data.unwrap()
    }


}