use url::Url;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::error::APIError;

use std::collections::HashMap;
use streamunordered::{StreamUnordered, StreamYield};
use pin_project::*;
use uuid::Uuid;

use futures::{prelude::*, StreamExt, SinkExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::model::{TradeEvent, BinanceSymbol, WebsocketEvent, BinanceWsResponse};
use crate::connect::{StoredStream};
use crate::connect::ExchangeSettings;

#[pin_project]
#[derive(Default)]
pub struct BinanceWs {
    subscriptions: HashMap<Uuid, usize>,
    tokens: HashMap<usize, Uuid>,
    #[pin]
    streams: StreamUnordered<StoredStream>,
}

const TRADE: &'static str = "trade";
impl BinanceWs {
    pub fn new() -> Self {
        BinanceWs::default()
    }
    pub fn parse_message(msg: Message) -> Result<WebsocketEvent, APIError> {
        match msg {
            Message::Text(msg) => {
                if msg.find(TRADE) != None {
                    let res = serde_json::from_str::<BinanceWsResponse>(&msg)?;
                    return Ok(WebsocketEvent::OneTrade(res.data));
                }
            }
            Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => (),
            Message::Close(_) => {
                return Err(APIError::Other(
                    format!("Websocket closed!")
                ));
            }
        }
        Ok(WebsocketEvent::None)
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
    async fn subscribe(&mut self, symbols: BinanceSymbol) -> Result<Uuid, APIError> {
        let url = Url::parse(&WEBSOCKET_BINANCE_URL).unwrap();
        let (ws_stream, _) = connect_async(url).await?;

        let (mut sink, read) = ws_stream.split();

        let sub = ExchangeSettings::from(symbols);

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
                                .and_then(|m| BinanceWs::parse_message(m)),
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
