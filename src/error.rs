use crate::error::other_err::{BinanceContentError, BinanceMiscError};

pub mod other_err {
    use std::collections::HashMap;

    use serde::*;
    use serde_json::Value;

    #[derive(thiserror::Error, Debug, Serialize, Deserialize)]
    #[error("({:?}) {:?}\n{:?}", code, msg, extra)]
    pub struct BinanceContentError {
        pub code: i16,
        pub msg: String,

        #[serde(flatten)]
        pub extra: HashMap<String, Value>,
    }

    #[derive(thiserror::Error, Debug)]
    #[error("{:?}", msg)]
    pub struct BinanceMiscError {
        msg: String
    }

    impl From<String> for BinanceMiscError {
        fn from(msg: String) -> Self {
            BinanceMiscError { msg }
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum BinanceErr {
    Serde(#[from] serde_json::error::Error),

    Websocket(#[from] tokio_tungstenite::tungstenite::Error),

    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    HTTP(#[from] reqwest::Error),

    IoError(#[from] std::io::Error),

    SystemTimeError(#[from] std::time::SystemTimeError),

    ParseFloatError(#[from] std::num::ParseFloatError),

    BinanceContentError(#[from] BinanceContentError),

    Other(#[from] BinanceMiscError),
}

impl BinanceErr {
    pub fn get_fmt_error(&mut self) -> String {
        match self {
            BinanceErr::Serde(e) => format!("{}", e),
            BinanceErr::Websocket(e) => format!("{}", e),
            BinanceErr::InvalidHeaderValue(e) => format!("{}", e),
            BinanceErr::HTTP(e) => format!("{}", e),
            BinanceErr::IoError(e) => format!("{}", e),
            BinanceErr::SystemTimeError(e) => format!("{}", e),
            BinanceErr::ParseFloatError(e) => format!("{}", e),
            BinanceErr::BinanceContentError(e) => format!("{}", e),
            BinanceErr::Other(e) => format!("{}", e),
        }
    }
}
