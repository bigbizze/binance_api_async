use serde_json::from_str;
use uuid::Uuid;

use crate::client::*;
use crate::error::*;
use crate::model::*;
use crate::websocket::*;

static USER_DATA_STREAM: &str = "/api/v3/userDataStream";


pub struct UserStream {
    pub client: Client,
    pub recv_window: u64,
    pub ws: Option<Websocket>,
}

#[async_trait::async_trait]
pub trait UserStreamAsync {
    async fn subscribe(&mut self) -> Result<Uuid, BinanceErr>;
    fn unsubscribe(&mut self, uuid: Uuid) -> Option<StoredStream>;
}

#[async_trait::async_trait]
impl UserStreamAsync for UserStream {
    async fn subscribe(&mut self) -> Result<Uuid, BinanceErr> {
        match self.start().await? {
            UserDataStream { listen_key } => {
                let mut ws = Websocket::new();
                let id = ws.subscribe(WebsocketStreamType::UserStream(listen_key)).await?;
                self.ws = Some(ws);
                Ok(id)
            }
        }
    }

    fn unsubscribe(&mut self, uuid: Uuid) -> Option<StoredStream> {
        if let Some(ws) = &mut self.ws {
            ws.unsubscribe(uuid)
        } else {
            None
        }
    }
}

impl UserStream {
    pub async fn start(&self) -> Result<UserDataStream, BinanceErr> {
        let data = self.client.post(USER_DATA_STREAM).await?;
        let user_data_stream: UserDataStream = from_str(data.as_str())?;
        Ok(user_data_stream)
    }

    pub async fn keep_alive(&self, listen_key: &str) -> Result<Success, BinanceErr> {
        let data = self.client.put(USER_DATA_STREAM, listen_key).await?;
        let success: Success = from_str(data.as_str())?;
        Ok(success)
    }

    pub async fn close(&self, listen_key: &str) -> Result<Success, BinanceErr> {
        let data = self.client.delete(USER_DATA_STREAM, listen_key).await?;
        let success: Success = from_str(data.as_str())?;
        Ok(success)
    }
}
