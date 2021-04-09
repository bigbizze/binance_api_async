use crate::futures::io::Error;
use std::collections::HashMap;
use serde_json::Value;
use serde::*;
use std::fmt::Display;
use crate::failure::_core::fmt::Formatter;
use crate::failure::_core::num::ParseFloatError;

#[derive(Debug, Deserialize)]
pub struct BinanceContentError {
    pub code: i16,
    pub msg: String,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}
impl Display for BinanceContentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let extra = if !self.extra.is_empty() {
            format!(" :: {}", serde_json::to_string_pretty(&self.extra).unwrap())
        } else {
            format!("")
        };
        write!(f, "({}) {}{}", self.code, self.msg, extra)
    }
}

#[derive(Fail, Debug)]
pub enum BinanceErr {
    #[fail(display = "Serde issue parsing error {}", _0)]
    Serde(#[fail(cause)] serde_json::Error),
    #[fail(display = "Websocket error {}", _0)]
    Websocket(#[fail(cause)] tokio_tungstenite::tungstenite::Error),
    #[fail(display = "REST Call error {}", _0)]
    HTTP(#[fail(cause)] reqwest::Error),
    #[fail(display = "Binance Content error {}", _0)]
    BinanceContent(BinanceContentError),
    #[fail(display = "IoError error {}", _0)]
    IoError(#[fail(cause)] std::io::Error),
    #[fail(display = "Other issue {}", _0)]
    Other(String),
    #[fail(display = "Invalid Header value {}", _0)]
    InvalidHeaderValue(#[fail(cause)] reqwest::header::InvalidHeaderValue),
    #[fail(display = "System Time error {}", _0)]
    SystemTimeError(#[fail(cause)] std::time::SystemTimeError),
    #[fail(display = "Parse Float error {}", _0)]
    ParseFloatError(#[fail(cause)] std::num::ParseFloatError)

}

impl BinanceErr {}

impl From<reqwest::Error> for BinanceErr {
    fn from(err: reqwest::Error) -> Self {
        BinanceErr::HTTP(err)
    }
}

impl From<serde_json::Error> for BinanceErr {
    fn from(err: serde_json::Error) -> Self {
        BinanceErr::Serde(err)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for BinanceErr {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        BinanceErr::Websocket(err)
    }
}

impl From<std::io::Error> for BinanceErr {
    fn from(err: Error) -> Self {
        BinanceErr::IoError(err)
    }
}

impl From<reqwest::header::InvalidHeaderValue> for BinanceErr {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        BinanceErr::InvalidHeaderValue(err)
    }
}

impl From<std::time::SystemTimeError> for BinanceErr {
    fn from(err: std::time::SystemTimeError) -> Self {
        BinanceErr::SystemTimeError(err)
    }
}

impl From<std::num::ParseFloatError> for BinanceErr {
    fn from(err: ParseFloatError) -> Self {
        BinanceErr::ParseFloatError(err)
    }
}
