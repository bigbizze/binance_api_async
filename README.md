# Rust Binance API Async/Await Library w/ Tokio


### Websockets
*https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md*

Individual Trade / Book Ticker / Aggregated Trades / Partial Book Depth Stream / 24 Hour Ticker / Diff. Depth Stream
```rust
use binance_ws::websocket::*;
use binance_ws::futures::TryStreamExt;
use binance_ws::api::Binance;

async fn individual_trades() -> Result<(), BinanceErr> {

    let symbols = vec!["ETHBTC".into(), "ADABTC".into()];

    let mut binance_ws: Websocket = Binance::new(None, None);
    
    // ** Change the variant wrapping the input symbols passed to the subscribe function to change the stream type!
    // For e.g., to do aggregated trades instead:
    // let sub_id = binance_ws.subscribe(WebsocketStreamType::AggregatedTrades(symbols)).await?;
    let sub_id = binance_ws.subscribe(WebsocketStreamType::IndividualTrade(symbols)).await?;
    
    while let Some(event) = binance_ws.try_next().await.expect("Didn't receive next transmit") {
        match event {
            WebsocketEvent::IndividualTrade(data) => {
                println!("{}, {}, {}", data.price, data.symbol, data.qty);
            },
            // Other events that use same format:
            
            // WebsocketEvent::BookTicker(data) => {
            //     println!("{}, {}, {}", data.best_bid, data.symbol, data.update_id);
            // },
            // WebsocketEvent::AggregatedTrades(data) => {
            //     println!("{}, {}, {}", data.price, data.symbol, data.qty);
            // },
            // WebsocketEvent::PartialBookDepthStream(data) => {
            //     println!("{}, {}, {}", data.asks.len(), data.bids.len(), data.last_update_id);
            // },
            // WebsocketEvent::TwentyFourHourTicker(data) => {
            //     println!("{}, {}, {}", data.prev_close, data.best_ask_qty, data.event_time);
            // },
            // WebsocketEvent::DiffDepthStream(data) => {
            //     println!("{}, {}, {}", data.bids.len(), data.symbol, data.event_time);
            // },
            _ => {}
        }
    }
    
    binance_ws.unsubscribe(sub_id);
    
    Ok(())
}

````

*Kline*
```rust
use binance_ws::websocket::*;
use binance_ws::futures::TryStreamExt;
use binance_ws::api::Binance;

async fn individual_trades() -> Result<(), BinanceErr> {
    let symbols = vec!["ETHBTC".into(), "ADABTC".into()];

    let mut binance_ws: Websocket = Binance::new(None, None);
    let sub_id = binance_ws.subscribe(WebsocketStreamType::Kline { interval: Some(KlineInterval::Minutes(5)), symbols }).await?;

    while let Some(event) = binance_ws.try_next().await.expect("Didn't receive next transmit") {
        match event {
            WebsocketEvent::Kline(kline) => {
                println!("{}, {}, {}", kline.symbol, kline.event_time, kline.kline.high);
            },
            _ => {}
        }
    }

    binance_ws.unsubscribe(sub_id);
}
```

*Day Ticker All*
```rust
use binance_ws::websocket::*;
use binance_ws::futures::TryStreamExt;
use binance_ws::api::Binance;
use binance_ws::userstream::{UserStream, UserStreamAsync};

async fn individual_trades() -> Result<(), BinanceErr> {
    let mut binance_ws: Websocket = Binance::new(None, None);
    let sub_id = binance_ws.subscribe(WebsocketStreamType::DayTickerAll).await?;

    while let Some(event) = binance_ws.try_next().await.expect("Didn't receive next transmit") {
        match event {
            WebsocketEvent::DayTickerAll(many_ticker) => {
                for ticker in many_ticker {
                    println!("{}, {}, {}", ticker.best_ask_qty, ticker.high, ticker.close_time);
                }
            },
            _ => {}
        }
    }

    binance_ws.unsubscribe(sub_id);
}
```

### User Stream
*https://github.com/binance/binance-spot-api-docs/blob/master/user-data-stream.md*
```rust
async fn individual_trades() -> Result<(), BinanceErr> {
    let mut user_stream: UserStream = Binance::new(Some("<api-key>".into()), Some("<api-secret>".into()));
    let sub_id = user_stream.subscribe().await?;
    let mut binance_ws = user_stream.ws.as_mut().expect("You didn't subscribe!");
    while let Some(event) = binance_ws.try_next().await.expect("Didn't receive next transmit") {
        match event {
            WebsocketEvent::OrderUpdate(data) => {
                println!("{}, {}, {}", data.accumulated_qty_filled_trades, data.commission, data.qty_last_filled_trade)
            },
            WebsocketEvent::AccountUpdate(data) => {
                println!("{}", data.balance.first().unwrap().free)
            },
            WebsocketEvent::BalanceUpdate(data) => {
                println!("{}, {}, {}", data.asset, data.balance_delta, data.clear_time)
            }
            _ => {}
        }
    }

    user_stream.unsubscribe(sub_id);
}
```

### HTTP Requests
*https://github.com/binance/binance-spot-api-docs/blob/master/rest-api.md*

*(Virtually all of this is just altering a fork of https://docs.rs/crate/binance/0.12.3 and making it async)*

### Market
```rust
use binance::api::*;
use binance::market::*;

async fn market() -> Result<(), BinanceErr> {
    let market: Market = Binance::new(None, None);

    // Order book at default depth
    let depth = market.get_depth("BNBETH").await?;

    // Latest price for ALL symbols
    let all_prices = market.get_all_prices().await?;
    
    // Latest price for ONE symbol
    let price = market.get_price("BNBETH").await?;

    // Current average price for ONE symbol
    let avg_price = market.get_average_price("BNBETH").await?;

    // Best price/qty on the order book for ALL symbols
    let all_book_tickers = market.get_all_book_tickers().await?;

    // Best price/qty on the order book for ONE symbol
    let book_ticker = market.get_book_ticker("BNBETH").await?;

    // 24hr ticker price change statistics
    let twenty_four_hour_price = market.get_24h_price_stats("BNBETH").await?;

    // last 10 5min klines (candlesticks) for a symbol:
    let klines = market.get_klines("BNBETH", "5m", 10, None, None).await?;
    
    Ok(())
}
```

### Account
```rust
async fn account() -> Result<(), BinanceErr> {
    let api_key = Some("YOUR_API_KEY".into());
    let secret_key = Some("YOUR_SECRET_KEY".into());

    let account: Account = Binance::new(api_key, secret_key);

    let account = account.get_account().await?;

    let open_orders = account.get_open_orders("WTCETH").await?;

    let limit_buy = account.limit_buy("WTCETH", 10, 0.014000).await?;

    let market_buy = account.market_buy("WTCETH", 5).await?;

    let limit_sell = account.limit_sell("WTCETH", 10, 0.035000).await?;

    let market_sell = account.market_sell("WTCETH", 5).await?;

    let custom_order = account.custom_order("WTCETH", 9999, 0.0123, "SELL", "LIMIT", "IOC").await?;

    let order_id = 1_957_528;
    let order_status = account.order_status("WTCETH", order_id).await?;

    let cancelled_order = account.cancel_order("WTCETH", order_id).await?;

    let all_cancelled_orders = account.cancel_all_open_orders("WTCETH").await?;

    let balances = account.get_balance("KNC").await?;

    let trade_history = account.trade_history("WTCETH").await?;
}
```
