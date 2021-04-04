/*!
## Implemented functionality
- [x] `Order Book`
- [x] `Recent Trades List`
- [ ] `Old Trades Lookup (MARKET_DATA)`
- [x] `Compressed/Aggregate Trades List`
- [x] `Kline/Candlestick Data`
- [x] `Mark Price`
- [ ] `Get Funding Rate History (MARKET_DATA)`
- [x] `24hr Ticker Price Change Statistics`
- [x] `Symbol Price Ticker`
- [x] `Symbol Order Book Ticker`
- [x] `Get all Liquidation Orders`
- [x] `Open Interest`
- [ ] `Notional and Leverage Brackets (MARKET_DATA)`
- [ ] `Open Interest Statistics (MARKET_DATA)`
- [ ] `Top Trader Long/Short Ratio (Accounts) (MARKET_DATA)`
- [ ] `Top Trader Long/Short Ratio (Positions) (MARKET_DATA)`
- [ ] `Long/Short Ratio (MARKET_DATA)`
- [ ] `Taker Buy/Sell Volume (MARKET_DATA)`
*/

use crate::util::*;
use crate::binance_futures::model::*;
use crate::client::*;
use crate::error::*;
use std::collections::BTreeMap;
use serde_json::{Value, from_str};


// TODO
// Make enums for Strings
// Add limit parameters to functions
// Implement all functions

#[derive(Clone)]
pub struct FuturesMarket {
    pub client: Client,
    pub recv_window: u64,
}

impl FuturesMarket {
    // Order book (Default 100; max 1000)
    pub async fn get_depth<S>(&self, symbol: S) -> Result<OrderBook, APIError>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        let data = self.client.get("/fapi/v1/depth", &request).await?;

        let order_book: OrderBook = from_str(data.as_str())?;

        Ok(order_book)
    }

    pub async fn get_trades<S>(&self, symbol: S) -> Result<Trades, APIError>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        let data = self.client.get("/fapi/v1/trades", &request).await?;

        let trades: Trades = from_str(data.as_str())?;

        Ok(trades)
    }

    // TODO This may be incomplete, as it hasn't been tested
    pub async fn get_historical_trades<S1, S2, S3>(
        &self, symbol: S1, from_id: S2, limit: S3,
    ) -> Result<Trades, APIError>
    where
        S1: Into<String>,
        S2: Into<Option<u64>>,
        S3: Into<Option<u16>>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());

        // Add three optional parameters
        if let Some(lt) = limit.into() {
            parameters.insert("limit".into(), format!("{}", lt));
        }
        if let Some(fi) = from_id.into() {
            parameters.insert("fromId".into(), format!("{}", fi));
        }

        let request = build_signed_request(parameters, self.recv_window)?;

        let data = self
            .client
            .get_signed("/fapi/v1/historicalTrades", &request).await?;

        let trades: Trades = from_str(data.as_str())?;

        Ok(trades)
    }

    pub async fn get_agg_trades<S1, S2, S3, S4, S5>(
        &self, symbol: S1, from_id: S2, start_time: S3, end_time: S4, limit: S5,
    ) -> Result<AggTrades, APIError>
    where
        S1: Into<String>,
        S2: Into<Option<u64>>,
        S3: Into<Option<u64>>,
        S4: Into<Option<u64>>,
        S5: Into<Option<u16>>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());

        // Add three optional parameters
        if let Some(lt) = limit.into() {
            parameters.insert("limit".into(), format!("{}", lt));
        }
        if let Some(st) = start_time.into() {
            parameters.insert("startTime".into(), format!("{}", st));
        }
        if let Some(et) = end_time.into() {
            parameters.insert("endTime".into(), format!("{}", et));
        }
        if let Some(fi) = from_id.into() {
            parameters.insert("fromId".into(), format!("{}", fi));
        }

        let request = build_request(&parameters);

        let data = self.client.get("/fapi/v1/aggTrades", &request).await?;

        let aggtrades: AggTrades = from_str(data.as_str())?;

        Ok(aggtrades)
    }

    // Returns up to 'limit' klines for given symbol and interval ("1m", "5m", ...)
    // https://github.com/binance-exchange/binance-official-api-docs/blob/master/rest-api.md#klinecandlestick-data
    pub async fn get_klines<S1, S2, S3, S4, S5>(
        &self, symbol: S1, interval: S2, limit: S3, start_time: S4, end_time: S5,
    ) -> Result<KlineSummaries, APIError>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<Option<u16>>,
        S4: Into<Option<u64>>,
        S5: Into<Option<u64>>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("interval".into(), interval.into());

        // Add three optional parameters
        if let Some(lt) = limit.into() {
            parameters.insert("limit".into(), format!("{}", lt));
        }
        if let Some(st) = start_time.into() {
            parameters.insert("startTime".into(), format!("{}", st));
        }
        if let Some(et) = end_time.into() {
            parameters.insert("endTime".into(), format!("{}", et));
        }

        let request = build_request(&parameters);

        let data = self.client.get("/fapi/v1/klines", &request).await?;
        let parsed_data: Vec<Vec<Value>> = from_str(data.as_str())?;

        let klines = KlineSummaries::AllKlineSummaries(
            parsed_data
                .iter()
                .map(|row| KlineSummary {
                    open_time: to_i64(&row[0]),
                    open: to_f64(&row[1]),
                    high: to_f64(&row[2]),
                    low: to_f64(&row[3]),
                    close: to_f64(&row[4]),
                    volume: to_f64(&row[5]),
                    close_time: to_i64(&row[6]),
                    quote_asset_volume: to_f64(&row[7]),
                    number_of_trades: to_i64(&row[8]),
                    taker_buy_base_asset_volume: to_f64(&row[9]),
                    taker_buy_quote_asset_volume: to_f64(&row[10]),
                })
                .collect(),
        );
        Ok(klines)
    }

    // 24hr ticker price change statistics
    pub async fn get_24h_price_stats<S>(&self, symbol: S) -> Result<PriceStats, APIError>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        let data = self.client.get("/fapi/v1/ticker/24hr", &request).await?;

        let stats: PriceStats = from_str(data.as_str())?;

        Ok(stats)
    }

    // Latest price for ONE symbol.
    pub async fn get_price<S>(&self, symbol: S) -> Result<SymbolPrice, APIError>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        let data = self.client.get("/fapi/v1/ticker/price", &request).await?;
        let symbol_price: SymbolPrice = from_str(data.as_str())?;

        Ok(symbol_price)
    }

    // Symbols order book ticker
    // -> Best price/qty on the order book for ALL symbols.
    pub async fn get_all_book_tickers(&self) -> Result<BookTickers, APIError> {
        let data = self.client.get("/fapi/v1/ticker/bookTicker", "").await?;

        let book_tickers: BookTickers = from_str(data.as_str())?;

        Ok(book_tickers)
    }

    // -> Best price/qty on the order book for ONE symbol
    pub async fn get_book_ticker<S>(&self, symbol: S) -> Result<Tickers, APIError>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        let data = self.client.get("/fapi/v1/ticker/bookTicker", &request).await?;
        let ticker: Tickers = from_str(data.as_str())?;

        Ok(ticker)
    }

    pub async fn get_mark_prices(&self) -> Result<MarkPrices, APIError> {
        let data = self.client.get("/fapi/v1/premiumIndex", "").await?;

        let mark_prices: MarkPrices = from_str(data.as_str())?;

        Ok(mark_prices)
    }

    pub async fn get_all_liquidation_orders(&self) -> Result<LiquidationOrders, APIError> {
        let data = self.client.get("/fapi/v1/allForceOrders", "").await?;
        let liquidation_orders: LiquidationOrders = from_str(data.as_str())?;

        Ok(liquidation_orders)
    }

    pub async fn open_interest<S>(&self, symbol: S) -> Result<OpenInterest, APIError>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        let data = self.client.get("/fapi/v1/openInterest", &request).await?;
        let open_interest: OpenInterest = from_str(data.as_str())?;

        Ok(open_interest)
    }
}
