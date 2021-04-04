use crate::util::*;
use crate::model::*;
// use crate::client::*;
use crate::error::*;
use std::collections::BTreeMap;
use serde_json::from_str;
use crate::client::Client;
// use crate::error::APIError;

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
}

struct OrderRequest {
    pub symbol: String,
    pub qty: f64,
    pub price: f64,
    pub order_side: String,
    pub order_type: String,
    pub time_in_force: String,
}

impl Account {
    // Account Information
    pub async fn get_account(&self) -> Result<AccountInformation, APIError> {
        let parameters: BTreeMap<String, String> = BTreeMap::new();

        let request = build_signed_request(parameters, self.recv_window)?;
        let data = self.client.get_signed("/api/v3/account", &request).await?;
        let account_info: AccountInformation = from_str(data.as_str())?;

        Ok(account_info)
    }

    // Balance for ONE Asset
    pub async fn get_balance<S>(&self, asset: S) -> Result<Balance, APIError>
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
                Err(APIError::Other(format!("Asset not found")))
            }
            Err(e) => Err(e),
        }
    }

    // Current open orders for ONE symbol
    pub async fn get_open_orders<S>(&self, symbol: S) -> Result<Vec<Order>, APIError>
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
    pub async fn get_all_open_orders(&self) -> Result<Vec<Order>, APIError> {
        let parameters: BTreeMap<String, String> = BTreeMap::new();

        let request = build_signed_request(parameters, self.recv_window)?;
        let data = self.client.get_signed("/api/v3/openOrders", &request).await?;
        let order: Vec<Order> = from_str(data.as_str())?;

        Ok(order)
    }
    pub async fn cancel_all_open_orders_void<S>(&self, symbol: S) -> Result<(), APIError>
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
    pub async fn cancel_all_open_orders<S>(&self, symbol: S) -> Result<Vec<Order>, APIError>
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
    pub async fn order_status<S>(&self, symbol: S, order_id: u64) -> Result<Order, APIError>
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
    pub async fn test_order_status<S>(&self, symbol: S, order_id: u64) -> Result<(), APIError>
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
    pub async fn limit_buy<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<Transaction, APIError>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let buy: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            order_side: ORDER_SIDE_BUY.to_string(),
            order_type: ORDER_TYPE_LIMIT.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(buy).await;
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER, &request).await?;
        let transaction: Transaction = from_str(data.as_str())?;

        Ok(transaction)
    }

    /// Place a test limit order - BUY
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching engine.
    pub async fn test_limit_buy<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<(), APIError>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let buy: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            order_side: ORDER_SIDE_BUY.to_string(),
            order_type: ORDER_TYPE_LIMIT.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(buy).await;
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER_TEST, &request).await?;
        let _: TestResponse = from_str(data.as_str())?;

        Ok(())
    }

    // Place a LIMIT order - SELL
    pub async fn limit_sell<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<Transaction, APIError>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            order_side: ORDER_SIDE_SELL.to_string(),
            order_type: ORDER_TYPE_LIMIT.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(sell).await;
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER, &request).await?;
        let transaction: Transaction = from_str(data.as_str())?;

        Ok(transaction)
    }

    /// Place a test LIMIT order - SELL
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching engine.
    pub async fn test_limit_sell<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<(), APIError>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            order_side: ORDER_SIDE_SELL.to_string(),
            order_type: ORDER_TYPE_LIMIT.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(sell).await;
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER_TEST, &request).await?;
        let _: TestResponse = from_str(data.as_str())?;

        Ok(())
    }

    // Place a MARKET order - BUY
    pub async fn market_buy<S, F>(&self, symbol: S, qty: F) -> Result<Transaction, APIError>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let buy: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price: 0.0,
            order_side: ORDER_SIDE_BUY.to_string(),
            order_type: ORDER_TYPE_MARKET.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(buy).await;
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER, &request).await?;
        let transaction: Transaction = from_str(data.as_str())?;

        Ok(transaction)
    }

    /// Place a test MARKET order - BUY
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching engine.
    pub async fn test_market_buy<S, F>(&self, symbol: S, qty: F) -> Result<(), APIError>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let buy: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price: 0.0,
            order_side: ORDER_SIDE_BUY.to_string(),
            order_type: ORDER_TYPE_MARKET.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(buy).await;
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER_TEST, &request).await?;
        let _: TestResponse = from_str(data.as_str())?;

        Ok(())
    }

    // Place a MARKET order - SELL
    pub async fn market_sell<S, F>(&self, symbol: S, qty: F) -> Result<Transaction, APIError>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price: 0.0,
            order_side: ORDER_SIDE_SELL.to_string(),
            order_type: ORDER_TYPE_MARKET.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(sell).await;
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER, &request).await?;
        let transaction: Transaction = from_str(data.as_str())?;

        Ok(transaction)
    }

    /// Place a test MARKET order - SELL
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching engine.
    pub async fn test_market_sell<S, F>(&self, symbol: S, qty: F) -> Result<(), APIError>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price: 0.0,
            order_side: ORDER_SIDE_SELL.to_string(),
            order_type: ORDER_TYPE_MARKET.to_string(),
            time_in_force: TIME_IN_FORCE_GTC.to_string(),
        };
        let order = self.build_order(sell).await;
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
    ) -> Result<Transaction, APIError>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            order_side: order_side.into(),
            order_type: order_type.into(),
            time_in_force: execution_type.into(),
        };
        let order = self.build_order(sell).await;
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
    ) -> Result<(), APIError>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            order_side: order_side.into(),
            order_type: order_type.into(),
            time_in_force: execution_type.into(),
        };
        let order = self.build_order(sell).await;
        let request = build_signed_request(order, self.recv_window)?;
        let data = self.client.post_signed(API_V3_ORDER_TEST, &request).await?;
        let _: TestResponse = from_str(data.as_str())?;

        Ok(())
    }

    // Check an order's status
    pub async fn cancel_order<S>(&self, symbol: S, order_id: u64) -> Result<OrderCanceled, APIError>
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
    pub async fn test_cancel_order<S>(&self, symbol: S, order_id: u64) -> Result<(), APIError>
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
    pub async fn trade_history<S>(&self, symbol: S) -> Result<Vec<TradeHistory>, APIError>
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

    async fn build_order(&self, order: OrderRequest) -> BTreeMap<String, String> {
        let mut order_parameters: BTreeMap<String, String> = BTreeMap::new();

        order_parameters.insert("symbol".into(), order.symbol);
        order_parameters.insert("side".into(), order.order_side);
        order_parameters.insert("type".into(), order.order_type);
        order_parameters.insert("quantity".into(), order.qty.to_string());

        if order.price != 0.0 {
            order_parameters.insert("price".into(), order.price.to_string());
            order_parameters.insert("timeInForce".into(), order.time_in_force);
        }

        order_parameters
    }
}
