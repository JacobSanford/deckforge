use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use serde::{Deserialize, Serialize};

use crate::blockchain::deckchain::DeckChain;
use crate::error::{DeckForgeError, Result};

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
    rarity: HashMap<String, Rarity>,
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
    special: HashMap<String, Special>,
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

    pub fn from_file(series_file: &str) -> Result<Self> {
        let file = File::open(series_file)?;
        let reader = BufReader::new(file);
        let series: TradingCardSeries = serde_json::from_reader(reader)?;
        series.validate_series()?;
        Ok(series)
    }

    pub fn from_deckchain(deckchain: &DeckChain, series_id: String) -> Result<Self> {
        let series_value = deckchain.card_series_release(&series_id)?;
        let series: TradingCardSeries = serde_json::from_value(series_value)?;
        series.validate_series()?;
        Ok(series)
    }

    pub fn get_card_configs(&self) -> &[CardConfig] {
        &self.config.cards
    }

    pub fn get_mint_each(&self) -> u32 {
        self.config.distribution.mint.total
    }

    pub fn get_mint_total(&self) -> u32 {
        self.get_mint_each() * self.get_card_configs().len() as u32
    }

    pub fn get_specials(&self) -> &HashMap<String, Special> {
        &self.config.distribution.mint.special
    }

    pub fn get_mint_list(&self) -> Vec<Vec<String>> {
        let mint_ea = self.get_mint_each();

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
            mint_list.push(vec![String::new()]);
        }
        mint_list
    }

    pub fn validate_series(&self) -> Result<()> {
        self.validate_series_values()
    }

    pub fn validate_series_values(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(DeckForgeError::Validation { reason: Self::ERROR_NO_ID.to_string() });
        }

        if self.name.is_empty() {
            return Err(DeckForgeError::Validation { reason: Self::ERROR_NO_NAME.to_string() });
        }

        if self.description.is_empty() {
            return Err(DeckForgeError::Validation { reason: Self::ERROR_NO_DESCRIPTION.to_string() });
        }

        if self.config.cards.is_empty() {
            return Err(DeckForgeError::Validation { reason: Self::ERROR_NO_CARDS.to_string() });
        }

        if !self.rarity_order_is_sensible() {
            return Err(DeckForgeError::Validation { reason: Self::ERROR_RARITY_ORDER.to_string() });
        }

        if !self.rarity_ratios_are_sensible() {
            return Err(DeckForgeError::Validation { reason: Self::ERROR_RARITY_RATIO.to_string() });
        }

        if !self.total_cards_match() {
            return Err(DeckForgeError::Validation { reason: Self::ERROR_TOTAL_CARDS_MISMATCH.to_string() });
        }

        if !self.card_rarities_match_card_configs() {
            return Err(DeckForgeError::Validation { reason: Self::ERROR_CARD_RARITIES_MISMATCH.to_string() });
        }

        if !self.mint_count_is_reasonable() {
            return Err(DeckForgeError::Validation { reason: "You should mint at least one of each card".to_string() });
        }

        if !self.mint_specials_counts_are_sensible() {
            return Err(DeckForgeError::Validation { reason: "Mint special cards shouldn't exceed more than 1/2.5 of printed cards".to_string() });
        }

        if !self.mint_special_order_is_sensible() {
            return Err(DeckForgeError::Validation { reason: "Mint Special order is not sensible".to_string() });
        }
        Ok(())
    }

    pub fn mint_special_order_is_sensible(&self) -> bool {
        let mint_specials = self.get_mint_specials_as_sorted_vec();
        let mut last_special = (Self::MAXIMUM_RARITY_VALUE / 2) - 1;
        for (_key, special) in mint_specials.iter() {
            if *special > (last_special / 2) {
                return false;
            }
            last_special = *special;
        }
        true
    }

    pub fn mint_specials_counts_are_sensible(&self) -> bool {
        let mint_specials = self.get_mint_specials_specs();
        let mint_each = self.get_mint_each();
        let total_specials: u32 = mint_specials.values().sum();
        total_specials < (mint_each as f32 / 2.5) as u32
    }

    pub fn mint_count_is_reasonable(&self) -> bool {
        self.get_mint_each() > 0
    }

    pub fn rarity_ratios_are_sensible(&self) -> bool {
        let rarity_value_counts = self.rarity_spec_as_sorted_vec();
        let mut last_rarity = (Self::MAXIMUM_RARITY_VALUE / 2) - 1;
        for (_key, rarity) in rarity_value_counts.iter() {
            if *rarity > (last_rarity / 2) {
                return false;
            }
            last_rarity = *rarity;
        }
        true
    }

    pub fn rarity_order_is_sensible(&self) -> bool {
        let rarity_value_counts = self.rarity_spec_as_sorted_vec();
        let mut last_rarity = Self::MAXIMUM_RARITY_VALUE;
        for (_key, rarity) in rarity_value_counts.iter() {
            if *rarity > last_rarity {
                return false;
            }
            last_rarity = *rarity;
        }
        true
    }

    pub fn get_mint_specials_specs(&self) -> HashMap<String, u32> {
        self.config
            .distribution
            .mint
            .special
            .iter()
            .map(|(key, special)| (key.to_string(), special.items))
            .collect()
    }

    pub fn get_mint_specials_as_sorted_vec(&self) -> Vec<(String, u32)> {
        let mut vec: Vec<(String, u32)> = self.get_mint_specials_specs().into_iter().collect();
        vec.sort_by(|a, b| a.0.cmp(&b.0));
        vec
    }

    pub fn get_rarity_specs(&self) -> HashMap<u32, u32> {
        self.config
            .distribution
            .rarity
            .iter()
            .filter_map(|(key, rarity)| {
                key.parse::<u32>().ok().map(|k| (k, rarity.items))
            })
            .collect()
    }

    pub fn rarity_spec_as_sorted_vec(&self) -> Vec<(u32, u32)> {
        let mut vec: Vec<(u32, u32)> = self.get_rarity_specs().into_iter().collect();
        vec.sort_by(|a, b| a.0.cmp(&b.0));
        vec
    }

    pub fn total_cards_match(&self) -> bool {
        let total_card_configs = self.config.cards.len() as u32;

        let total_rarity_cards: u32 = self.config.distribution.rarity.values().map(|r| r.items).sum();

        total_card_configs == total_rarity_cards
    }

    pub fn card_rarities_match_card_configs(&self) -> bool {
        let mut card_rarity_counts = HashMap::new();
        for card_config in &self.config.cards {
            *card_rarity_counts.entry(card_config.rarity).or_insert(0) += 1;
        }

        let rarity_value_counts = self.get_rarity_specs();

        card_rarity_counts == rarity_value_counts
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use rand::seq::SliceRandom;
    use serde_json::Value;

    #[test]
    fn test_series_ok() {
        let series = test_series_data();
        let series_values_result = series.validate_series_values();
        assert!(series_values_result.is_ok());
    }

    #[test]
    fn test_series_no_id() {
        let mut series = test_series_data();
        series.id = "".to_string();
        let series_values_result = series.validate_series_values();
        assert!(series_values_result.is_err());
        let err_msg = series_values_result.err().unwrap().to_string();
        assert!(err_msg.contains(TradingCardSeries::ERROR_NO_ID), "got: {}", err_msg);
    }

    #[test]
    fn test_series_data_from_file() {
        let series = test_series_data();
        assert_eq!(series.id, "LEGACYDECK-1");
    }

    #[test]
    fn test_series_no_description() {
        let mut series = test_series_data();
        series.description = "".to_string();
        let series_values_result = series.validate_series_values();
        assert!(series_values_result.is_err());
        let err_msg = series_values_result.err().unwrap().to_string();
        assert!(err_msg.contains(TradingCardSeries::ERROR_NO_DESCRIPTION), "got: {}", err_msg);
    }

    #[test]
    fn test_series_no_name() {
        let mut series = test_series_data();
        series.name = "".to_string();
        let series_values_result = series.validate_series_values();
        assert!(series_values_result.is_err());
        let err_msg = series_values_result.err().unwrap().to_string();
        assert!(err_msg.contains(TradingCardSeries::ERROR_NO_NAME), "got: {}", err_msg);
    }

    #[test]
    fn test_series_no_cards() {
        let mut series = test_series_data();
        series.config.cards = Vec::new();
        let series_values_result = series.validate_series_values();
        assert!(series_values_result.is_err());
        let err_msg = series_values_result.err().unwrap().to_string();
        assert!(err_msg.contains(TradingCardSeries::ERROR_NO_CARDS), "got: {}", err_msg);
    }

    #[test]
    fn test_series_mismatch_total_cards() {
        let mut series = test_series_data();
        let number_to_remove = get_random_number(&series);
        series.config.cards.retain(|x| x.number != number_to_remove);

        let series_values_result = series.total_cards_match();
        assert!(!series_values_result);
    }

    #[test]
    fn test_series_mismatch_card_rarities() {
        let mut series = test_series_data();

        let rarities = get_card_rarities(&series);
        let rarity_to_change = rarities.choose(&mut rand::thread_rng()).unwrap();
        let rarity_to_set = rarities.iter().find(|&&x| x != *rarity_to_change).unwrap();

        for card in series.config.cards.iter_mut() {
            if card.rarity == *rarity_to_change {
                card.rarity = *rarity_to_set;
                break;
            }
        }

        let series_values_result = series.card_rarities_match_card_configs();
        assert!(!series_values_result);
    }

    #[test]
    fn test_rarity_order_is_sensible() {
        let mut series = test_series_data();

        let mut rarity_vec = series.rarity_spec_as_sorted_vec();
        let rarity_count = rarity_vec.len() as u32;

        for (key, _rarity) in rarity_vec.iter_mut() {
            *key = rarity_count - *key;
        }

        for (key, rarity) in rarity_vec.iter() {
            series.config.distribution.rarity.insert(key.to_string(), Rarity {
                name: key.to_string(),
                items: *rarity,
            });
        }

        let series_values_result = series.rarity_order_is_sensible();
        assert!(!series_values_result);
    }

    #[test]
    fn test_rarity_ratios_are_sensible() {
        let mut series = test_series_data();
        let mut last_rarity = 256;

        let mut rarity_vec = series.rarity_spec_as_sorted_vec();

        for (_key, rarity) in rarity_vec.iter_mut() {
            *rarity = (last_rarity / 2) + 1;
            last_rarity = *rarity;
        }

        for (key, rarity) in rarity_vec.iter() {
            series.config.distribution.rarity.insert(key.to_string(), Rarity {
                name: key.to_string(),
                items: *rarity,
            });
        }

        let series_values_result = series.rarity_ratios_are_sensible();
        assert!(!series_values_result);
    }

    #[test]
    fn test_mint_ea_is_sensible() {
        let mut series = test_series_data();
        series.config.distribution.mint.total = 0;
        let series_values_result = series.mint_count_is_reasonable();
        assert!(!series_values_result);
    }

    #[test]
    fn test_mint_specials_order_is_sensible() {
        let mut series = test_series_data();
        let mint_specials = series.get_mint_specials_as_sorted_vec();
        let mut last_special = (TradingCardSeries::MAXIMUM_RARITY_VALUE / 2) - 1;
        for (_key, special) in mint_specials.iter() {
            if *special > (last_special / 2) {
                let series_values_result = series.mint_special_order_is_sensible();
                assert!(!series_values_result);
                return;
            }
            last_special = *special;
        }
        let series_values_result = series.mint_special_order_is_sensible();
        assert!(series_values_result);
    }

    #[test]
    fn test_mint_specials_counts_are_sensible() {
        let mut series = test_series_data();
        let mint_each = series.get_mint_each();

        if let Some((_key, special)) = series.config.distribution.mint.special.iter_mut().next() {
            special.items = mint_each / 2;
        }

        let series_values_result = series.mint_specials_counts_are_sensible();
        assert!(!series_values_result);
    }

    #[test]
    fn test_mint_list() {
        let series = test_series_data();
        let mint_list = series.get_mint_list();
        let mint_ea = series.get_mint_each();

        let special_list = series.config.distribution.mint.special.clone();

        assert_eq!(mint_list.len() as u32, mint_ea);

        let mut prop_counter: HashMap<&str, u32> = HashMap::new();
        for props in mint_list.iter() {
            for prop in props.iter() {
                *prop_counter.entry(prop).or_insert(0) += 1;
            }
        }

        for special_item in special_list.iter() {
            let special = special_item.1;
            let special_name = &special.name;
            let special_items = special.items;

            let special_count = prop_counter.get(special_name.as_str()).unwrap_or(&0);
            assert_eq!(special_items, *special_count, "Special {} was minted {} times, expected {}", special_name, special_count, *special_count);
        }
    }

    #[test]
    fn test_get_mint_each() {
        let series = test_series_data();
        let mint_each = series.get_mint_each();
        assert_eq!(mint_each, 242_u32);
    }

    #[test]
    fn test_get_mint_total() {
        let series = test_series_data();
        let mint_total = series.get_mint_total();
        let mint_each = series.get_mint_each();
        let card_configs = series.get_card_configs();
        let desired_num_cards = mint_each * card_configs.len() as u32;
        assert_eq!(mint_total, desired_num_cards);
    }

    fn get_card_rarities(series: &TradingCardSeries) -> Vec<u32> {
        series.config.cards.iter().map(|c| c.rarity).collect()
    }

    fn get_random_number(series: &TradingCardSeries) -> u32 {
        let card = series.config.cards.choose(&mut rand::thread_rng());
        card.unwrap().number
    }

    pub fn test_series_json() -> Value {
        let series_file = "test/series.json".to_string();
        let file = File::open(series_file).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    }

    pub fn test_series_data() -> TradingCardSeries {
        let series_file = "test/series.json";
        TradingCardSeries::from_file(series_file).unwrap()
    }
}
