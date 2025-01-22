use serde::{Deserialize, Serialize};
use serde_json::Result;

use crate::card::series::CardConfig;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TradingCardReleaseSet {
    pub cards: Vec<TradingCard>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct TradingCard {
    series: String,
    number: u32,
    serial: String,
    title: String,
    description: String,
    rarity: u32,
    card_type: String,
    pub properties: Vec<String>,
}

impl TradingCard {
    pub fn from_json(json_data: &str, properties: Vec<String>) -> Result<Self> {
        let mut card: TradingCard = serde_json::from_str(json_data)?;
        card.properties = properties;
        Ok(card)
    }

    pub fn from_card_config(card_config: &CardConfig, properties: Vec<String>, series: String, serial: String) -> Self {
        TradingCard {
            series,
            number: card_config.number.clone(),
            serial,
            title: card_config.title.clone(),
            description: card_config.description.clone(),
            rarity: card_config.rarity,
            card_type: card_config.card_type.clone(),
            properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::blockchain::testing::init_test_data_dir;

    #[tokio::test]
    async fn test_release_set() {
        let series_file = "test/series.json";
        
    }

}