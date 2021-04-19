#![feature(backtrace)]
#[macro_use]
pub extern crate failure;
pub extern crate futures;

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

