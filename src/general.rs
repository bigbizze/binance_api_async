use serde_json::from_str;

use crate::client::*;
use crate::error::*;
use crate::error::other_err::BinanceMiscError;
use crate::model::*;

#[derive(Clone)]
pub struct General {
    pub client: Client,
}

impl General {
    // Test connectivity
    pub async fn ping(&self) -> Result<String, BinanceErr> {
        self.client.get("/api/v3/ping", "").await?;

        Ok("pong".into())
    }

    // Check server time
    pub async fn get_server_time(&self) -> Result<ServerTime, BinanceErr> {
        let data: String = self.client.get("/api/v3/time", "").await?;

        let server_time: ServerTime = from_str(data.as_str())?;

        Ok(server_time)
    }

    // Obtain exchange information
    // - Current exchange trading rules and symbol information
    pub async fn exchange_info(&self) -> Result<ExchangeInformation, BinanceErr> {
        let data: String = self.client.get("/api/v3/exchangeInfo", "").await?;

        let info: ExchangeInformation = from_str(data.as_str())?;

        Ok(info)
    }

    // Get Symbol information
    pub async fn get_symbol_info<S>(&self, symbol: S) -> Result<Symbol, BinanceErr>
        where
            S: Into<String>,
    {
        let upper_symbol = symbol.into().to_uppercase();
        match self.exchange_info().await {
            Ok(info) => {
                for item in info.symbols {
                    if item.symbol == upper_symbol {
                        return Ok(item);
                    }
                }
                Err(BinanceErr::Other(BinanceMiscError::from(format!("Symbol not found"))))
            }
            Err(e) => Err(e),
        }
    }
}
