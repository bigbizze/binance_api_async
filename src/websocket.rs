use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{prelude::*, SinkExt, stream::SplitStream, StreamExt};
use pin_project::*;
use serde::*;
use streamunordered::{StreamUnordered, StreamYield};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_tungstenite::WebSocketStream;
use url::Url;
use uuid::Uuid;

use crate::error::*;
use crate::error::other_err::BinanceMiscError;
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
    pub fn map_symbols_to_stream_params(stream_type: WebsocketStreamType) -> Vec<String> {
        match stream_type {
            WebsocketStreamType::IndividualTrade(s) => {
                s.into_iter().map(|e| format!("{}@trade", e.to_lowercase())).collect()
            }
            WebsocketStreamType::AggregatedTrades(s) => {
                s.into_iter().map(|e| format!("{}@aggTrade", e.to_lowercase())).collect()
            }
            WebsocketStreamType::TwentyFourHourTicker(s) => {
                s.into_iter().map(|e| format!("{}@24hrTicker", e.to_lowercase())).collect()
            }
            WebsocketStreamType::DayTickerAll => {
                vec![format!("!ticker@arr")]
            }
            WebsocketStreamType::Kline { symbols: s, mut interval } => {
                let interval = interval.format_interval();
                s.into_iter().map(|e| format!("{}@kline_{}", e.to_lowercase(), interval)).collect()
            }
            WebsocketStreamType::PartialBookDepthStream(s) => {
                s.into_iter().map(|e| format!("{}@lastUpdateId", e.to_lowercase())).collect()
            }
            WebsocketStreamType::DiffDepthStream(s) => {
                s.into_iter().map(|e| format!("{}@depthUpdate", e.to_lowercase())).collect()
            }
            WebsocketStreamType::BookTicker(s) => {
                s.into_iter().map(|e| format!("{}@bookTicker", e.to_lowercase())).collect()
            }
            _ => vec![]
        }
    }
}

impl From<WebsocketStreamType> for ExchangeSettings {
    fn from(f: WebsocketStreamType) -> Self {
        let params = ExchangeSettings::map_symbols_to_stream_params(f);
        ExchangeSettings {
            method: METHOD,
            params,
            id: 1,
        }
    }
}

pub enum KlineInterval {
    Minutes(u16),
    Hours(u16),
    Days(u16),
    Weeks(u16),
    Months(u16),
    None,
}

impl KlineInterval {
    pub fn format_interval(&mut self) -> String {
        match self {
            KlineInterval::Minutes(t) => format!("{}m", t),
            KlineInterval::Hours(t) => format!("{}h", t),
            KlineInterval::Days(t) => format!("{}d", t),
            KlineInterval::Weeks(t) => format!("{}w", t),
            KlineInterval::Months(t) => format!("{}M", t),
            KlineInterval::None => format!("1m")
        }
    }
}

pub enum WebsocketStreamType {
    BookTicker(Vec<String>),
    AggregatedTrades(Vec<String>),
    IndividualTrade(Vec<String>),
    PartialBookDepthStream(Vec<String>),
    TwentyFourHourTicker(Vec<String>),
    Kline { symbols: Vec<String>, interval: KlineInterval },
    DiffDepthStream(Vec<String>),
    DayTickerAll,
    UserStream(String),
}

pub enum UserDataStreamType {
    AccountUpdate,
    OrderUpdate,
    BalanceUpdate,
}

impl WebsocketStreamType {
    pub fn first(&mut self) -> String {
        match self {
            WebsocketStreamType::AggregatedTrades(s) => s[0].clone(),
            WebsocketStreamType::IndividualTrade(s) => s[0].clone(),
            WebsocketStreamType::PartialBookDepthStream(s) => s[0].clone(),
            WebsocketStreamType::TwentyFourHourTicker(s) => s[0].clone(),
            WebsocketStreamType::DayTickerAll => format!("!ticker@arr"),
            WebsocketStreamType::Kline { symbols: s, .. } => s[0].clone(),
            WebsocketStreamType::DiffDepthStream(s) => s[0].clone(),
            WebsocketStreamType::BookTicker(s) => s[0].clone(),
            WebsocketStreamType::UserStream(listen_key) => listen_key.clone()
        }
    }
    pub fn len(&mut self) -> usize {
        match self {
            WebsocketStreamType::AggregatedTrades(s) => s.len(),
            WebsocketStreamType::IndividualTrade(s) => s.len(),
            WebsocketStreamType::PartialBookDepthStream(s) => s.len(),
            WebsocketStreamType::TwentyFourHourTicker(s) => s.len(),
            WebsocketStreamType::DayTickerAll => 1,
            WebsocketStreamType::Kline { symbols: s, .. } => s.len(),
            WebsocketStreamType::DiffDepthStream(s) => s.len(),
            WebsocketStreamType::BookTicker(s) => s.len(),
            WebsocketStreamType::UserStream(_) => 1
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Debug, Deserialize)]
pub enum WebsocketEvent {
    AggregatedTrades(TradesEvent),
    IndividualTrade(OneTradeEvent),
    TwentyFourHourTicker(DayTickerEvent),
    DayTickerAll(Vec<DayTickerEvent>),
    Kline(KlineEvent),
    DiffDepthStream(DepthOrderBookEvent),
    PartialBookDepthStream(OrderBook),
    BookTicker(BookTickerEvent),

    AccountUpdate(AccountUpdateEvent),
    OrderUpdate(OrderTradeEvent),
    BalanceUpdate(BalanceUpdateEvent),
    None,
}

const STREAM: &'static str = "stream";

const AGGREGATED_TRADE: &'static str = "aggTrade";
const INDIVIDUAL_TRADE: &'static str = "trade";
const DIFF_DEPTH_ORDER_BOOK: &'static str = "depthUpdate";
const KLINE: &'static str = "kline";
const PARTIAL_DEPTH_ORDER_BOOK: &'static str = "lastUpdateId";
const TWENTY_FOUR_HOUR_TICKER: &'static str = "24hrTicker";

const ACCOUNT_UPDATE: &'static str = "outboundAccountInfo";
const ORDER_UPDATE: &'static str = "executionReport";
const BALANCE_UPDATE: &'static str = "balanceUpdate";

#[pin_project]
#[derive(Default)]
pub struct Websocket {
    subscribe_single_value: Option<String>,
    subscriptions: HashMap<Uuid, usize>,
    tokens: HashMap<usize, Uuid>,
    #[pin]
    streams: StreamUnordered<StoredStream>,
}

impl Websocket {
    pub fn new() -> Self {
        Websocket::default()
    }
    pub fn parse_response_type(&mut self, msg: &str) -> Result<WebsocketEvent, BinanceErr> {
        let value: serde_json::Value = serde_json::from_str(msg)?;
        return Ok(if msg.find(STREAM) != None {
            if value["data"] != serde_json::Value::Null {
                let data = format!("{}", value["data"]);
                self.parse_response_type(&data)?
            } else {
                return Err(BinanceErr::Other(BinanceMiscError::from(format!("Websocket closed!"))));
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
        } else if msg.find(INDIVIDUAL_TRADE) != None {
            let trade: OneTradeEvent = serde_json::from_str(msg)?;
            WebsocketEvent::IndividualTrade(trade)
        } else if msg.find(ACCOUNT_UPDATE) != None {
            let account_update: AccountUpdateEvent = serde_json::from_str(msg)?;
            WebsocketEvent::AccountUpdate(account_update)
        } else if msg.find(BALANCE_UPDATE) != None {
            let balance_update: BalanceUpdateEvent = serde_json::from_str(msg)?;
            WebsocketEvent::BalanceUpdate(balance_update)
        } else if msg.find(ORDER_UPDATE) != None {
            let order_trade: OrderTradeEvent = serde_json::from_str(msg)?;
            WebsocketEvent::OrderUpdate(order_trade)
        } else if msg.find(AGGREGATED_TRADE) != None {
            let trade: TradesEvent = serde_json::from_str(msg)?;
            WebsocketEvent::AggregatedTrades(trade)
        } else if msg.find(TWENTY_FOUR_HOUR_TICKER) != None {
            if let Some(single_value) = &self.subscribe_single_value {
                if single_value == "!ticker@arr" {
                    let trades: Vec<DayTickerEvent> = serde_json::from_str(msg)?;
                    return Ok(WebsocketEvent::DayTickerAll(trades));
                }
            }
            let trades: DayTickerEvent = serde_json::from_str(msg)?;
            WebsocketEvent::TwentyFourHourTicker(trades)
        } else if msg.find(KLINE) != None {
            let kline: KlineEvent = serde_json::from_str(msg)?;
            WebsocketEvent::Kline(kline)
        } else if msg.find(PARTIAL_DEPTH_ORDER_BOOK) != None {
            let partial_orderbook: OrderBook = serde_json::from_str(msg)?;
            WebsocketEvent::PartialBookDepthStream(partial_orderbook)
        } else if msg.find(DIFF_DEPTH_ORDER_BOOK) != None {
            let depth_orderbook: DepthOrderBookEvent = serde_json::from_str(msg)?;
            WebsocketEvent::DiffDepthStream(depth_orderbook)
        } else {
            WebsocketEvent::None
        });
    }
    pub fn parse_message(&mut self, msg: Message) -> Result<WebsocketEvent, BinanceErr> {
        return match msg {
            Message::Text(msg) => self.parse_response_type(&msg),
            Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => Ok(WebsocketEvent::None),
            Message::Close(_) => Err(BinanceErr::Other(BinanceMiscError::from(format!("Websocket closed!"))))
        };
    }
}

const WEBSOCKET_BINANCE_URL: &'static str = "wss://stream.binance.com:9443/stream";

#[async_trait::async_trait]
pub trait WebsocketAsync {
    async fn subscribe(&mut self, endpoint: WebsocketStreamType) -> Result<Uuid, BinanceErr>;
    fn unsubscribe(&mut self, uuid: Uuid) -> Option<StoredStream>;
}

#[async_trait::async_trait]
impl WebsocketAsync for Websocket {
    async fn subscribe(&mut self, mut stream_type: WebsocketStreamType) -> Result<Uuid, BinanceErr> {
        self.subscribe_single_value = if stream_type.len() == 1 {
            Some(stream_type.first())
        } else {
            None
        };
        let url = Url::parse(&WEBSOCKET_BINANCE_URL).unwrap();
        let (ws_stream, _) = connect_async(url).await?;

        let (mut sink, read) = ws_stream.split();

        if let WebsocketStreamType::UserStream(_) = stream_type {} else {
            let sub = ExchangeSettings::from(stream_type);
            sink.send(Message::Text(serde_json::to_string(&sub)?)).await?;
        }

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

impl Stream for Websocket {
    type Item = Result<WebsocketEvent, BinanceErr>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.as_mut().project().streams.poll_next(cx) {
            Poll::Ready(Some((y, _))) => match y {
                StreamYield::Item(item) => {
                    Poll::Ready({
                        Some(
                            item.map_err(BinanceErr::Websocket)
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

    use crate::error::BinanceErr;
    use crate::websocket::*;

    fn correct_symbol(res: WebsocketEvent) -> bool {
        if let WebsocketEvent::IndividualTrade(trade) = res {
            trade.symbol == "ADABTC" || trade.symbol == "ETHBTC"
        } else {
            false
        }
    }

    async fn test_binance_ws() -> Result<(), BinanceErr> {
        let mut binance_ws = Websocket::new();
        let endpoints = vec!["ETHBTC".into(), "ADABTC".into()];
        let sub_uuid = binance_ws.subscribe(WebsocketStreamType::IndividualTrade(endpoints)).await?;
        for _ in 0..5 {
            let res = binance_ws.try_next().await.expect("Didn't receive next transmit");
            let res = res.expect("Got no match for trade type!");
            if let WebsocketEvent::None = res {
                continue;
            }
            assert!(correct_symbol(res));
        }
        binance_ws.unsubscribe(sub_uuid);
        Ok(())
    }

    #[tokio::main]
    #[test]
    async fn it_works() {
        test_binance_ws().await.ok();
    }
}
