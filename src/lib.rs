#[macro_use]
pub extern crate failure;
pub mod binance_ws;
pub mod error;
pub mod model;
mod connect;

pub extern crate futures;
#[cfg(test)]
mod tests {
    use crate::binance_ws::{BinanceWs, BinanceWsAsync};
    use futures::TryStreamExt;
    use crate::error::APIError;
    use crate::model::{BinanceSymbol, WebsocketEvent};

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
            if i == 0 {
                assert_eq!(res, WebsocketEvent::None);
            } else {
                assert!(correct_symbol(res));
            }
        }
        Ok(())
    }
    #[tokio::main]
    #[test]
    async fn it_works() {
        test_binance_ws().await;
    }
}
