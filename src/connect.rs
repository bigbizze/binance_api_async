use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use futures::{prelude::*, stream::SplitStream, StreamExt, SinkExt};
use crate::model::{TradeEvent, BinanceSymbol};
use serde::*;
type WSStream = WebSocketStream<tokio_tungstenite::stream::Stream<TcpStream, tokio_native_tls::TlsStream<TcpStream>>>;
pub type StoredStream = SplitStream<WSStream>;

const METHOD: &'static str = "SUBSCRIBE";

#[derive(Serialize, Deserialize)]
pub struct ExchangeSettings {
    method: &'static str,
    params: Vec<String>,
    id: i64,
}

impl ExchangeSettings {
    pub fn make_trade_param(endpoint: String) -> String {
        format!("{}@trade", endpoint.to_lowercase())
    }
}

impl From<BinanceSymbol> for ExchangeSettings {
    fn from(f: BinanceSymbol) -> Self {
        let params: Vec<String> = match f {
            BinanceSymbol::Multiple(endpoints) => {
                endpoints.into_iter().map(|e| ExchangeSettings::make_trade_param(e)).collect()
            }
            BinanceSymbol::One(endpoint) => {
                vec![ExchangeSettings::make_trade_param(endpoint)]
            }
        };
        ExchangeSettings {
            method: METHOD,
            params,
            id: 1

        }
    }
}
