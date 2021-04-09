use crate::util::*;
use crate::model::*;

use crate::error::*;
use std::collections::{BTreeMap, HashMap};
use serde_json::from_str;
use crate::client::Client;
use crate::general::General;
use crate::api::Binance;

static ORDER_TYPE_LIMIT: &str = "LIMIT";
static ORDER_TYPE_MARKET: &str = "MARKET";
static ORDER_SIDE_BUY: &str = "BUY";
static ORDER_SIDE_SELL: &str = "SELL";
static TIME_IN_FORCE_GTC: &str = "GTC";

static API_V3_ORDER: &str = "/api/v3/order";

/// Endpoint for test orders.
///
/// Orders issued to this endpoint are validated, but not sent into the matching engine.
static API_V3_ORDER_TEST: &str = "/api/v3/order/test";

#[derive(Clone)]
pub struct Account {
    pub client: Client,
    pub recv_window: u64,
    pub(crate) precisions_map: Option<HashMap<String, SymbolInfo>>,
}

struct OrderRequest {
    pub symbol: String,
    pub qty: String,
    pub price: Option<String>,
    pub order_side: String,
    pub order_type: String,
    pub time_in_force: String,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub(crate) quantity_precision: usize,
    pub(crate) price_precision: usize,
}

pub type SymbolPrecision = HashMap<String, SymbolInfo>;

impl Account {
    // Account Information
    pub async fn get_account(&self) -> Result<AccountInformation, BinanceErr> {
        let parameters: BTreeMap<String, String> = BTreeMap::new();

        let request = build_signed_request(parameters, self.recv_window)?;
        let data = self.client.get_signed("/api/v3/account", &request).await?;
        let account_info: AccountInformation = from_str(data.as_str())?;

        Ok(account_info)
    }

    // Balance for ONE Asset
    pub async fn get_balance<S>(&self, asset: S) -> Result<Balance, BinanceErr>
    where
        S: Into<String>,
    {
        match self.get_account().await {
            Ok(account) => {
                let cmp_asset = asset.into();
                for balance in account.balances {
                    if balance.asset == cmp_asset {
                        return Ok(balance);
                    }
                }
                Err(BinanceErr::Other(format!("Asset not found")))
            }
            Err(e) => Err(e),
        }
    }

    // Current open orders for ONE symbol
    pub async fn get_open_orders<S>(&self, symbol: S) -> Result<Vec<Order>, BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());

        let request = build_signed_request(parameters, self.recv_window)?;
        let data = self.client.get_signed("/api/v3/openOrders", &request).await?;
        let order: Vec<Order> = from_str(data.as_str())?;

        Ok(order)
    }

    // All current open orders
    pub async fn get_all_open_orders(&self) -> Result<Vec<Order>, BinanceErr> {
        let parameters: BTreeMap<String, String> = BTreeMap::new();

        let request = build_signed_request(parameters, self.recv_window)?;
        let data = self.client.get_signed("/api/v3/openOrders", &request).await?;
        let order: Vec<Order> = from_str(data.as_str())?;

        Ok(order)
    }
    pub async fn cancel_all_open_orders_void<S>(&self, symbol: S) -> Result<(), BinanceErr>
        where
            S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_signed_request(parameters, self.recv_window)?;
        self.client.delete_signed("/api/v3/openOrders", &request).await?;
        Ok(())
    }
    // Cancel all open orders for ONE symbol
    pub async fn cancel_all_open_orders<S>(&self, symbol: S) -> Result<Vec<Order>, BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_signed_request(parameters, self.recv_window)?;
        let data = self.client.delete_signed("/api/v3/openOrders", &request).await?;
        let order: Vec<Order> = from_str(data.as_str())?;

        Ok(order)
    }

    // Check an order's status
    pub async fn order_status<S>(&self, symbol: S, order_id: u64) -> Result<Order, BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("orderId".into(), order_id.to_string());

        let request = build_signed_request(parameters, self.recv_window)?;
        let data = self.client.get_signed(API_V3_ORDER, &request).await?;
        let order: Order = from_str(data.as_str())?;

        Ok(order)
    }

    /// Place a test status order
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching engine.
    pub async fn test_order_status<S>(&self, symbol: S, order_id: u64) -> Result<(), BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("orderId".into(), order_id.to_string());

        let request = build_signed_request(parameters, self.recv_window)?;
        let data = self.client.get_signed(API_V3_ORDER_TEST, &request).await?;
        let _: TestResponse = from_str(data.as_str())?;

        Ok(())
    }

    // Place a LIMIT order - BUY
    pub async fn limit_buy<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<Transaction, BinanceErr>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let symbol = symbol.into();
        let (qty, price) = self.sometimes_precise_quantity_and_price(&symbol, qty.into(), Some(price));
        let buy: OrderRequest = OrderRequest {
            symbol,
            qty,
            price,
            order_side: ORDER_SIDE_BUY.to_string(),
            order_type: ORDER_TYPE_LIMIT.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER, &request).await?;
        let transaction: Transaction = from_str(data.as_str())?;

        Ok(transaction)
    }

    /// Place a test limit order - BUY
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching engine.
    pub async fn test_limit_buy<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<(), BinanceErr>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let symbol = symbol.into();
        let (qty, price) = self.sometimes_precise_quantity_and_price(&symbol, qty.into(), Some(price));
        let buy: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty,
            price,
            order_side: ORDER_SIDE_BUY.to_string(),
            order_type: ORDER_TYPE_LIMIT.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER_TEST, &request).await?;
        let _: TestResponse = from_str(data.as_str())?;

        Ok(())
    }

    // Place a LIMIT order - SELL
    pub async fn limit_sell<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<Transaction, BinanceErr>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let symbol = symbol.into();
        let (qty, price) = self.sometimes_precise_quantity_and_price(&symbol, qty.into(), Some(price));
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty,
            price,
            order_side: ORDER_SIDE_SELL.to_string(),
            order_type: ORDER_TYPE_LIMIT.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER, &request).await?;
        let transaction: Transaction = from_str(data.as_str())?;

        Ok(transaction)
    }

    /// Place a test LIMIT order - SELL
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching engine.
    pub async fn test_limit_sell<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<(), BinanceErr>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let symbol = symbol.into();
        let (qty, price) = self.sometimes_precise_quantity_and_price(&symbol, qty.into(), Some(price));
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty,
            price,
            order_side: ORDER_SIDE_SELL.to_string(),
            order_type: ORDER_TYPE_LIMIT.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER_TEST, &request).await?;
        let _: TestResponse = from_str(data.as_str())?;

        Ok(())
    }

    // Place a MARKET order - BUY
    pub async fn market_buy<S, F>(&self, symbol: S, qty: F) -> Result<Transaction, BinanceErr>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let symbol = symbol.into();
        let (qty, price) = self.sometimes_precise_quantity_and_price(&symbol, qty.into(), None);
        let buy: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty,
            price,
            order_side: ORDER_SIDE_BUY.to_string(),
            order_type: ORDER_TYPE_MARKET.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER, &request).await?;
        let transaction: Transaction = from_str(data.as_str())?;

        Ok(transaction)
    }

    /// Place a test MARKET order - BUY
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching engine.
    pub async fn test_market_buy<S, F>(&self, symbol: S, qty: F) -> Result<(), BinanceErr>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let symbol = symbol.into();
        let (qty, price) = self.sometimes_precise_quantity_and_price(&symbol, qty.into(), None);
        let buy: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty,
            price,
            order_side: ORDER_SIDE_BUY.to_string(),
            order_type: ORDER_TYPE_MARKET.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER_TEST, &request).await?;
        let _: TestResponse = from_str(data.as_str())?;

        Ok(())
    }

    // Place a MARKET order - SELL
    pub async fn market_sell<S, F>(&self, symbol: S, qty: F) -> Result<Transaction, BinanceErr>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let symbol = symbol.into();
        let (qty, price) = self.sometimes_precise_quantity_and_price(&symbol, qty.into(), None);
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty,
            price,
            order_side: ORDER_SIDE_SELL.to_string(),
            order_type: ORDER_TYPE_MARKET.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER, &request).await?;
        let transaction: Transaction = from_str(data.as_str())?;

        Ok(transaction)
    }

    /// Place a test MARKET order - SELL
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching engine.
    pub async fn test_market_sell<S, F>(&self, symbol: S, qty: F) -> Result<(), BinanceErr>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let symbol = symbol.into();
        let (qty, price) = self.sometimes_precise_quantity_and_price(&symbol, qty.into(), None);
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty,
            price,
            order_side: ORDER_SIDE_SELL.to_string(),
            order_type: ORDER_TYPE_MARKET.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER_TEST, &request).await?;
        let _: TestResponse = from_str(data.as_str())?;

        Ok(())
    }


    /// Place a custom order
    pub async fn custom_order<S, F>(
        &self,
        symbol: S,
        qty: F,
        price: f64,
        order_side: S,
        order_type: S,
        execution_type: S,
    ) -> Result<Transaction, BinanceErr>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let symbol = symbol.into();
        let (qty, price) = self.sometimes_precise_quantity_and_price(&symbol, qty.into(), Some(price));
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty,
            price,
            order_side: order_side.into(),
            order_type: order_type.into(),
            time_in_force: execution_type.into(),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER, &request).await?;
        let transaction: Transaction = from_str(data.as_str())?;

        Ok(transaction)
    }


    /// Place a test custom order
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching engine.
    pub async fn test_custom_order<S, F>(
        &self,
        symbol: S,
        qty: F,
        price: f64,
        order_side: S,
        order_type: S,
        execution_type: S,
    ) -> Result<(), BinanceErr>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let symbol = symbol.into();
        let (qty, price) = self.sometimes_precise_quantity_and_price(&symbol, qty.into(), Some(price));
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty,
            price,
            order_side: order_side.into(),
            order_type: order_type.into(),
            time_in_force: execution_type.into(),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER_TEST, &request).await?;
        let _: TestResponse = from_str(data.as_str())?;

        Ok(())
    }

    // Check an order's status
    pub async fn cancel_order<S>(&self, symbol: S, order_id: u64) -> Result<OrderCanceled, BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("orderId".into(), order_id.to_string());

        let request = build_signed_request(parameters, self.recv_window)?;
        let data = self.client.delete_signed(API_V3_ORDER, &request).await?;
        let order_canceled: OrderCanceled = from_str(data.as_str())?;

        Ok(order_canceled)
    }

    /// Place a test cancel order
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching engine.
    pub async fn test_cancel_order<S>(&self, symbol: S, order_id: u64) -> Result<(), BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("orderId".into(), order_id.to_string());

        let request = build_signed_request(parameters, self.recv_window)?;
        let data = self.client.delete_signed(API_V3_ORDER_TEST, &request).await?;
        let _: TestResponse = from_str(data.as_str())?;

        Ok(())
    }

    // Trade history
    pub async fn trade_history<S>(&self, symbol: S) -> Result<Vec<TradeHistory>, BinanceErr>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());

        let request = build_signed_request(parameters, self.recv_window)?;
        let data = self.client.get_signed("/api/v3/myTrades", &request).await?;
        let trade_history: Vec<TradeHistory> = from_str(data.as_str())?;

        Ok(trade_history)
    }

    fn build_order(&self, order: OrderRequest) -> BTreeMap<String, String> {
        let mut order_parameters: BTreeMap<String, String> = BTreeMap::new();

        order_parameters.insert("symbol".into(), order.symbol);
        order_parameters.insert("side".into(), order.order_side);
        order_parameters.insert("type".into(), order.order_type);
        order_parameters.insert("quantity ".into(), order.qty);
        if let Some(order_price) = order.price {
            order_parameters.insert("price".into(), order_price);
            order_parameters.insert("timeInForce".into(), order.time_in_force);
        }

        order_parameters
    }

    fn sometimes_precise_quantity_and_price(&self, symbol: &String, qty: f64, price: Option<f64>) -> (String, Option<String>) {
        if let Some(p_map) = &self.precisions_map {
            if let Some(precision) = p_map.get(symbol) {
                let precise_qty = format!("{:.1$}", qty, precision.quantity_precision);
                let precise_price = if let Some(price) = price {
                    if price != 0.0 {
                        Some(format!("{:.1$}", price, precision.price_precision))
                    } else {
                        None
                    }
                } else {
                    None
                };
                return (precise_qty, precise_price);
            }
        }
        (qty.to_string(), if let Some(price) = price {
            Some(price.to_string())
        } else {
            None
        })
    }

    fn get_one_symbol_info(symbol_info: &Symbol) -> Result<SymbolInfo, BinanceErr> {
        let price_precision = symbol_info.quote_precision as usize;
        if &symbol_info.filters.len() < &3 {
            return Err(BinanceErr::Other(format!("Binance exchange info didn't return enough filters!")))
        }
        let lot_size = &symbol_info.filters[2];
        let quantity_precision: usize = match lot_size {
            Filters::LotSize { step_size, .. } => Ok(-step_size.parse::<f64>()?.log10() as usize),
            _ => Err(BinanceErr::Other(format!("There was no lot size!"))),
        }?;
        Ok(SymbolInfo {
            price_precision,
            quantity_precision,
        })
    }

    async fn get_exchange_info_binance() -> Result<SymbolPrecision, BinanceErr> {
        let client: General = Binance::new(None, None);
        let exchange_info = client.exchange_info().await?;
        let mut exchange_info_map = HashMap::<String, SymbolInfo>::new();
        let symbols = exchange_info.symbols.iter().filter(|s| s.status != "BREAK");
        for symbol in symbols {
            exchange_info_map.insert(
                symbol.symbol.to_string(),
                Account::get_one_symbol_info(symbol)?,
            );
        }
        Ok(exchange_info_map)
    }

    pub async fn load_symbol_precisions(&mut self) -> Result<(), BinanceErr> {
        let exchange_info = Account::get_exchange_info_binance().await?;
        self.precisions_map = Some(exchange_info);
        Ok(())
    }
}
