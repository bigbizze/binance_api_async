#[macro_use]
pub extern crate failure;
pub mod error;
pub mod model;
pub mod websocket;
pub mod client;
pub mod account;
pub mod util;
pub mod general;
pub mod market;
pub mod userstream;
pub mod binance_futures;
pub mod api;

pub extern crate futures;
#[cfg(test)]
mod tests {
    use futures::TryStreamExt;
    use crate::error::BinanceErr;

    use crate::websocket::*;
    use crate::api::Binance;
    use crate::market::Market;

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

    async fn test_binance2() {
        let binance: Market = Binance::new(None, None);
        let price = binance.get_price(format!("ETHBTC")).await.unwrap().price;
        println!("{}", price);
    }
    #[tokio::main]
    #[test]
    async fn it_works() {
        test_binance2().await;
        // test_binance_ws().await.ok();
    }
}
