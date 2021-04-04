use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use crate::model::*;
use serde::*;
use serde::*;
use url::Url;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::error::APIError;

use std::collections::HashMap;
use streamunordered::{StreamUnordered, StreamYield};
use pin_project::*;
use uuid::Uuid;

use futures::{prelude::*, StreamExt, SinkExt, stream::SplitStream};
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::model::*;


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

pub enum BinanceSymbol {
    Multiple(Vec<String>),
    One(String)
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Debug, Deserialize)]
pub enum WebsocketEvent {
    AccountUpdate(AccountUpdateEvent),
    OrderTrade(OrderTradeEvent),
    Trade(TradesEvent),
    OneTrade(OneTradeEvent),
    OrderBook(OrderBook),
    DayTicker(DayTickerEvent),
    DayTickerAll(Vec<DayTickerEvent>),
    Kline(KlineEvent),
    DepthOrderBook(DepthOrderBookEvent),
    BookTicker(BookTickerEvent),
    None
}
#[pin_project]
#[derive(Default)]
pub struct BinanceWs {
    subscribe_single_value: Option<String>,
    subscriptions: HashMap<Uuid, usize>,
    tokens: HashMap<usize, Uuid>,
    #[pin]
    streams: StreamUnordered<StoredStream>,
}

const OUTBOUND_ACCOUNT_INFO: &'static str = "outboundAccountInfo";
const EXECUTION_REPORT: &'static str = "executionReport";
const DEPTH_ORDER_BOOK: &'static str = "depthUpdate";
const KLINE: &'static str = "kline";
const PARTIAL_ORDER_BOOK: &'static str = "lastUpdateId";
const AGGREGATED_TRADE: &'static str = "aggTrade";
const DAYTICKER: &'static str = "24hrTicker";
const STREAM: &'static str = "stream";
const TRADE: &'static str = "trade";

impl BinanceWs {
    pub fn new() -> Self {
        BinanceWs::default()
    }
    pub fn parse_response_type(&mut self, msg: &str) -> Result<WebsocketEvent, APIError> {
        let value: serde_json::Value = serde_json::from_str(msg)?;
        return Ok(if msg.find(STREAM) != None {
            if value["data"] != serde_json::Value::Null {
                let data = format!("{}", value["data"]);
                self.parse_response_type(&data)?
            } else {
                return Err(APIError::Other(format!("Websocket closed!")))
            }
        } else if value["u"] != serde_json::Value::Null
            && value["s"] != serde_json::Value::Null
            && value["b"] != serde_json::Value::Null
            && value["B"] != serde_json::Value::Null
            && value["a"] != serde_json::Value::Null
            && value["A"] != serde_json::Value::Null
        {
            let book_ticker: BookTickerEvent = serde_json::from_str(msg)?;
            WebsocketEvent::BookTicker(book_ticker)
        } else if msg.find(TRADE) != None {
            let trade: OneTradeEvent = serde_json::from_str(msg)?;
            WebsocketEvent::OneTrade(trade)
        } else if msg.find(OUTBOUND_ACCOUNT_INFO) != None {
            let account_update: AccountUpdateEvent = serde_json::from_str(msg)?;
            WebsocketEvent::AccountUpdate(account_update)
        } else if msg.find(EXECUTION_REPORT) != None {
            let order_trade: OrderTradeEvent = serde_json::from_str(msg)?;
            WebsocketEvent::OrderTrade(order_trade)
        } else if msg.find(AGGREGATED_TRADE) != None {
            let trade: TradesEvent = serde_json::from_str(msg)?;
            WebsocketEvent::Trade(trade)
        } else if msg.find(DAYTICKER) != None {
            if let Some(single_value) = &self.subscribe_single_value {
                if single_value == "!ticker@arr" {
                    let trades: Vec<DayTickerEvent> = serde_json::from_str(msg)?;
                    return Ok(WebsocketEvent::DayTickerAll(trades))
                }
            }
            let trades: DayTickerEvent = serde_json::from_str(msg)?;
            WebsocketEvent::DayTicker(trades)
        } else if msg.find(KLINE) != None {
            let kline: KlineEvent = serde_json::from_str(msg)?;
            WebsocketEvent::Kline(kline)
        } else if msg.find(PARTIAL_ORDER_BOOK) != None {
            let partial_orderbook: OrderBook = serde_json::from_str(msg)?;
            WebsocketEvent::OrderBook(partial_orderbook)
        } else if msg.find(DEPTH_ORDER_BOOK) != None {
            let depth_orderbook: DepthOrderBookEvent = serde_json::from_str(msg)?;
            WebsocketEvent::DepthOrderBook(depth_orderbook)
        } else {
            WebsocketEvent::None
        });
    }
    pub fn parse_message(&mut self, msg: Message) -> Result<WebsocketEvent, APIError> {
        return match msg {
            Message::Text(msg) => self.parse_response_type(&msg),
            Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => Ok(WebsocketEvent::None),
            Message::Close(_) => Err(APIError::Other(format!("Websocket closed!")))
        };

    }
}

const WEBSOCKET_BINANCE_URL: &'static str = "wss://stream.binance.com:9443/stream";
#[async_trait::async_trait]
pub trait BinanceWsAsync {
    async fn subscribe(&mut self, endpoint: BinanceSymbol) -> Result<Uuid, APIError>;
    fn unsubscribe(&mut self, uuid: Uuid) -> Option<StoredStream>;
}

#[async_trait::async_trait]
impl BinanceWsAsync for BinanceWs {
    async fn subscribe(&mut self, endpoints: BinanceSymbol) -> Result<Uuid, APIError> {
        self.subscribe_single_value = if let BinanceSymbol::One(s) = &endpoints {
            Some(s.clone())
        } else {
            None
        };
        let url = Url::parse(&WEBSOCKET_BINANCE_URL).unwrap();
        let (ws_stream, _) = connect_async(url).await?;

        let (mut sink, read) = ws_stream.split();

        let sub = ExchangeSettings::from(endpoints);

        sink.send(Message::Text(serde_json::to_string(&sub)?)).await?;

        let uuid = Uuid::new_v4();
        let token = self.streams.insert(read);
        self.subscriptions.insert(uuid, token);
        self.tokens.insert(token, uuid);
        Ok(uuid)
    }

    fn unsubscribe(&mut self, uuid: Uuid) -> Option<StoredStream> {
        let streams = Pin::new(&mut self.streams);
        self.subscriptions
            .get(&uuid)
            .and_then(|token| StreamUnordered::take(streams, *token))
    }
}

impl Stream for BinanceWs {
    type Item = Result<WebsocketEvent, APIError>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.as_mut().project().streams.poll_next(cx) {
            Poll::Ready(Some((y, _))) => match y {
                StreamYield::Item(item) => {
                    // let heartbeat = self.heartbeats.get_mut(&token).unwrap();
                    Poll::Ready({
                        Some(
                            item.map_err(APIError::Websocket)
                                .and_then(|m| self.parse_message(m)),
                        )
                    })
                }
                StreamYield::Finished(_) => Poll::Pending,
            },
            Poll::Ready(None) => panic!("No Stream Subscribed"),
            Poll::Pending => Poll::Pending,
        }
    }
}

/** Websocket trade stream test
*/
#[cfg(test)]
mod tests {
    use futures::TryStreamExt;
    use crate::error::APIError;
    use crate::model::*;
    use crate::websocket::*;
    use crate::api::Binance;
    use crate::account::Account;

    fn correct_symbol(res: WebsocketEvent) -> bool {
        if let WebsocketEvent::OneTrade(trade) = res {
            trade.symbol == "ADABTC" || trade.symbol == "ETHBTC"
        } else {
            false
        }
    }

    async fn test_binance_ws() -> Result<(), APIError> {
        let mut binance_ws = BinanceWs::new();
        let endpoints = vec!["ETHBTC".into(), "ADABTC".into()];
        let sub_uuid = binance_ws.subscribe(BinanceSymbol::Multiple(endpoints)).await?;
        for i in 0..5 {
            let res = binance_ws.try_next().await.expect("Didn't receive next transmit");
            let res = res.expect("Got no match for trade type!");
            if let WebsocketEvent::None = res {
                continue;
            }
            assert!(correct_symbol(res));
        }
        Ok(())
    }

    #[tokio::main]
    #[test]
    async fn it_works() {
        test_binance_ws().await;
    }
}
