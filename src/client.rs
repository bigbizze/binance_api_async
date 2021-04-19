use hex::encode as hex_encode;
use hmac::{Hmac, Mac, NewMac};
use reqwest::{Response, StatusCode};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use sha2::Sha256;

use crate::error::BinanceErr;
use crate::error::other_err::*;

#[derive(Clone)]
pub struct Client {
    api_key: String,
    secret_key: String,
    host: String,
}

impl Client {
    pub fn new(api_key: Option<String>, secret_key: Option<String>, host: String) -> Self {
        Client {
            api_key: api_key.unwrap_or_else(|| "".into()),
            secret_key: secret_key.unwrap_or_else(|| "".into()),
            host,
        }
    }

    pub async fn get_signed(&self, endpoint: &str, request: &str) -> Result<String, BinanceErr> {
        let url = self.sign_request(endpoint, request).await;
        let client = reqwest::Client::new();
        let response = client
            .get(url.as_str())
            .headers(self.build_headers(true)?)
            .send()
            .await?;

        self.handler(response).await
    }

    pub async fn post_signed(&self, endpoint: &str, request: &str) -> Result<String, BinanceErr> {
        let url = self.sign_request(endpoint, request).await;
        let client = reqwest::Client::new();
        let response = client
            .post(url.as_str())
            .headers(self.build_headers(true)?)
            .send()
            .await?;

        self.handler(response).await
    }

    pub async fn delete_signed(&self, endpoint: &str, request: &str) -> Result<String, BinanceErr> {
        let url = self.sign_request(endpoint, request).await;
        let client = reqwest::Client::new();
        let response = client
            .delete(url.as_str())
            .headers(self.build_headers(true)?)
            .send()
            .await?;

        self.handler(response).await
    }

    pub async fn get(&self, endpoint: &str, request: &str) -> Result<String, BinanceErr> {
        let mut url: String = format!("{}{}", self.host, endpoint);
        if !request.is_empty() {
            url.push_str(format!("?{}", request).as_str());
        }

        let response = reqwest::get(url.as_str()).await?;

        self.handler(response).await
    }

    pub async fn post(&self, endpoint: &str) -> Result<String, BinanceErr> {
        let url: String = format!("{}{}", self.host, endpoint);

        let client = reqwest::Client::new();
        let response = client
            .post(url.as_str())
            .headers(self.build_headers(false)?)
            .send()
            .await?;

        self.handler(response).await
    }

    pub async fn put(&self, endpoint: &str, listen_key: &str) -> Result<String, BinanceErr> {
        let url: String = format!("{}{}", self.host, endpoint);
        let data: String = format!("listenKey={}", listen_key);

        let client = reqwest::Client::new();
        let response = client
            .put(url.as_str())
            .headers(self.build_headers(false)?)
            .body(data)
            .send()
            .await?;

        self.handler(response).await
    }

    pub async fn delete(&self, endpoint: &str, listen_key: &str) -> Result<String, BinanceErr> {
        let url: String = format!("{}{}", self.host, endpoint);
        let data: String = format!("listenKey={}", listen_key);

        let client = reqwest::Client::new();
        let response = client
            .delete(url.as_str())
            .headers(self.build_headers(false)?)
            .body(data)
            .send()
            .await?;

        self.handler(response).await
    }

    async fn sign_request(&self, endpoint: &str, request: &str) -> String {
        let mut signed_key = Hmac::<Sha256>::new_varkey(self.secret_key.as_bytes()).unwrap();
        signed_key.update(request.as_bytes());
        let signature = hex_encode(signed_key.finalize().into_bytes());
        let request_body: String = format!("{}&signature={}", request, signature);
        let url: String = format!("{}{}?{}", self.host, endpoint, request_body);

        url
    }

    fn build_headers(&self, content_type: bool) -> Result<HeaderMap, BinanceErr> {
        let mut custom_headers = HeaderMap::new();

        custom_headers.insert(USER_AGENT, HeaderValue::from_static("binance-rs"));
        if content_type {
            custom_headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            );
        }
        custom_headers.insert(
            HeaderName::from_static("x-mbx-apikey"),
            HeaderValue::from_str(self.api_key.as_str())?,
        );

        Ok(custom_headers)
    }

    async fn handler(&self, response: Response) -> Result<String, BinanceErr> {
        match response.status() {
            StatusCode::OK => {
                Ok(response.text().await?)
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                Err(BinanceErr::from_str(format!("Internal Server Error")))
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                Err(BinanceErr::from_str(format!("Service Unavailable")))
            }
            StatusCode::UNAUTHORIZED => {
                Err(BinanceErr::from_str(format!("Unauthorized")))
            }
            StatusCode::BAD_REQUEST => {
                Err(BinanceErr::BinanceContentError(response.json::<BinanceContentError>().await?))
            }
            s => {
                Err(BinanceErr::from_str(format!("Received response: {:?}", s)))
            }
        }
    }
}
