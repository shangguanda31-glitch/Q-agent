use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, error, warn};
use futures_util::StreamExt;
use url::Url;

use super::types::OneBotEvent;

pub async fn connect(
    ws_url: &str,
    token: &str,
    tx: Arc<broadcast::Sender<OneBotEvent>>,
) -> anyhow::Result<()> {
    let mut url = Url::parse(ws_url)?;
    url.query_pairs_mut().append_pair("access_token", token);
    let url_str = url.to_string();

    info!("Connecting to NapCatQQ WebSocket: {}", url_str);
    let (ws_stream, _) = connect_async(&url_str).await?;
    info!("WebSocket connected");

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<OneBotEvent>(&text) {
                    Ok(event) => {
                        let _ = tx.send(event);
                    }
                    Err(e) => {
                        warn!("Failed to parse event: {} | raw: {}", e, &text[..text.len().min(200)]);
                    }
                }
            }
            Ok(Message::Ping(_)) => {}
            Ok(Message::Pong(_)) => {}
            Ok(Message::Close(frame)) => {
                info!("WebSocket closed: {:?}", frame);
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
