use crate::account::*;
use crate::market::*;
use crate::general::*;
use crate::binance_futures::general::*;
use crate::binance_futures::market::*;
use crate::userstream::*;
use crate::client::*;
use crate::websocket::Websocket;

static API_HOST: &str = "https://api.binance.com";
static FAPI_HOST: &str = "https://fapi.binance.com";

//#[derive(Clone)]
pub trait Binance {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self;
}

impl Binance for General {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> General {
        General {
            client: Client::new(api_key, secret_key, API_HOST.to_string()),
        }
    }
}

impl Binance for Account {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Account {
        Account {
            client: Client::new(api_key, secret_key, API_HOST.to_string()),
            recv_window: 5000,
        }
    }
}

impl Binance for Market {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Market {
        Market {
            client: Client::new(api_key, secret_key, API_HOST.to_string()),
            recv_window: 5000,
        }
    }
}

impl Binance for UserStream {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> UserStream {
        UserStream {
            client: Client::new(api_key, secret_key, API_HOST.to_string()),
            recv_window: 5000,
            ws: None
        }
    }
}

impl Binance for Websocket {
    fn new(_: Option<String>, _: Option<String>) -> Self {
        Websocket::new()
    }
}

// *****************************************************
//              Binance Futures API
// *****************************************************

impl Binance for FuturesGeneral {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> FuturesGeneral {
        FuturesGeneral {
            client: Client::new(api_key, secret_key, FAPI_HOST.to_string()),
        }
    }
}

impl Binance for FuturesMarket {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> FuturesMarket {
        FuturesMarket {
            client: Client::new(api_key, secret_key, FAPI_HOST.to_string()),
            recv_window: 5000,
        }
    }
}
