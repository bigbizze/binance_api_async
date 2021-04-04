use serde::*;

#[derive(Serialize, Deserialize)]
pub struct BinanceWsResponse {
    pub stream: String,
    pub data: TradeEvent,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TradeEvent {
    #[serde(rename = "e")]
    pub trade_info_e: String,
    #[serde(rename = "E")]
    pub event_time: u128,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "t")]
    pub trade_time: u128,
    #[serde(rename = "p")]
    pub price: String,
    #[serde(rename = "q")]
    pub qty: String,
    pub b: i64,
    pub a: i64,
    #[serde(rename = "T")]
    pub t: i64,
    #[serde(rename = "m")]
    pub trade_info_m: bool,
    #[serde(rename = "M")]
    pub m: bool,
}

pub enum BinanceSymbol {
    Multiple(Vec<String>),
    One(String)
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Debug)]
pub enum WebsocketEvent {
    OneTrade(TradeEvent),
    None
}

impl PartialEq for WebsocketEvent {
    fn eq(&self, other: &Self) -> bool {
        let other_none = if let WebsocketEvent::None = other {
            true
        } else {
            false
        };
        let self_none = if let WebsocketEvent::None = self {
            true
        } else {
            false
        };
        other_none == self_none
    }
}
