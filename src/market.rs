use std::collections::BTreeMap;

use serde_json::{from_str, Value};

use crate::client::*;
use crate::error::*;
use crate::model::*;
use crate::util::*;

#[derive(Clone)]
pub struct Market {
    pub client: Client,
    pub recv_window: u64,
}

/// Market Data endpoints
impl Market {
    /// Order book (Default 100; max 100)
    pub async fn get_depth<S>(&self, symbol: S) -> Result<OrderBook, BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        let data = self.client.get("/api/v3/depth", &request).await?;

        let order_book: OrderBook = from_str(data.as_str())?;

        Ok(order_book)
    }

    /// Order book with a custom depth
    /// Supported depths by binance: 5, 10, 20, 50, 100, 500, 1000, 5000
    pub async fn get_custom_depth<S>(&self, symbol: S, limit: u16) -> Result<OrderBook, BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("limit".into(), limit.to_string());
        let request = build_request(&parameters);

        let data = self.client.get("/api/v3/depth", &request).await?;

        let order_book: OrderBook = from_str(data.as_str())?;

        Ok(order_book)
    }

    /// Latest price for ALL symbols.
    pub async fn get_all_prices(&self) -> Result<Prices, BinanceErr> {
        let data = self.client.get("/api/v3/ticker/price", "").await?;

        let prices: Prices = from_str(data.as_str())?;

        Ok(prices)
    }

    /// Latest price for ONE symbol.
    pub async fn get_price<S>(&self, symbol: S) -> Result<SymbolPrice, BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        let data = self.client.get("/api/v3/ticker/price", &request).await?;
        let symbol_price: SymbolPrice = from_str(data.as_str())?;

        Ok(symbol_price)
    }

    /// Average price for ONE symbol.
    pub async fn get_average_price<S>(&self, symbol: S) -> Result<AveragePrice, BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        let data = self.client.get("/api/v3/avgPrice", &request).await?;
        let average_price: AveragePrice = from_str(data.as_str())?;

        Ok(average_price)
    }

    /// Symbols order book ticker
    /// -> Best price/qty on the order book for ALL symbols.
    pub async fn get_all_book_tickers(&self) -> Result<BookTickers, BinanceErr> {
        let data = self.client.get("/api/v3/ticker/bookTicker", "").await?;

        let book_tickers: BookTickers = from_str(data.as_str())?;

        Ok(book_tickers)
    }

    /// -> Best price/qty on the order book for ONE symbol
    pub async fn get_book_ticker<S>(&self, symbol: S) -> Result<Tickers, BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        let data = self
            .client
            .get("/api/v3/ticker/bookTicker", &request)
            .await?;
        let ticker: Tickers = from_str(data.as_str())?;

        Ok(ticker)
    }

    /// 24hr ticker price change statistics
    pub async fn get_24h_price_stats<S>(&self, symbol: S) -> Result<PriceStats, BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        let data = self.client.get("/api/v3/ticker/24hr", &request).await?;

        let stats: PriceStats = from_str(data.as_str())?;

        Ok(stats)
    }

    // Returns up to 'limit' klines for given symbol and interval ("1m", "5m", ...)
    // https://github.com/binance-exchange/binance-official-api-docs/blob/master/rest-api.md#klinecandlestick-data
    pub async fn get_klines<S1, S2, S3, S4, S5>(
        &self,
        symbol: S1,
        interval: S2,
        limit: S3,
        start_time: S4,
        end_time: S5,
    ) -> Result<KlineSummaries, BinanceErr>
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

        let data = self.client.get("/api/v3/klines", &request).await?;
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
}
