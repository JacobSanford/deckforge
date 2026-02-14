use serde::{Deserialize, Serialize};

use crate::card::series::CardConfig;

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
    pub fn from_card_config(
        card_config: &CardConfig,
        properties: Vec<String>,
        series: String,
        serial: String,
    ) -> Self {
        TradingCard {
            series,
            number: card_config.number,
            serial,
            title: card_config.title.clone(),
            description: card_config.description.clone(),
            rarity: card_config.rarity,
            card_type: card_config.card_type.clone(),
            properties,
        }
    }
}
