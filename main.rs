use futures_util::{SinkExt, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::{time::{Duration, Instant}};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use warp::Filter;

#[tokio::main]
async fn main() {
    // HARDCODED TOKEN
    let token = "MTQ0MDc1Mjk2MzA0MjY3NjkzMQ.GGv7mw.3na5P8FNgTJplyglOzfEXwK8FMhSb8F7Dl_2l0";

    // 1. RENDER HEALTH CHECK (Port 10000)
    // Prevents Render from failing the deploy
    tokio::spawn(async move {
        let health = warp::path("healthz").map(|| "OK");
        warp::serve(health).run(([0, 0, 0, 0], 10000)).await;
    });

    let gateway = "wss://gateway.discord.gg/?v=10&encoding=json";
    let (mut ws_stream, _) = connect_async(gateway).await.expect("Failed to connect");

    // 2. IDENTIFY: Handshake for 24/7 message access
    let identify = json!({
        "op": 2,
        "d": {
            "token": token,
            "properties": { "$os": "linux", "$browser": "chrome", "$device": "chrome" }
        }
    }).to_string();
    ws_stream.send(Message::Text(identify)).await.unwrap();

    let (mut tx, mut rx) = ws_stream.split();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(40)).await; // Heartbeat to stay connected
            let _ = tx.send(Message::Text(json!({"op": 1, "d": null}).to_string())).await;
        }
    });

    println!("[!] Sniper Active in Virginia...");

    while let Some(msg) = rx.next().await {
        if let Ok(Message::Text(text)) = msg {
            let data = text.as_bytes();
            // HOT PATH: Byte scanning for maximum speed
            if let Some(pos) = data.windows(13).position(|w| w == b"discord.gift/") {
                let start = Instant::now();
                let code = String::from_utf8_lossy(&data[pos+13..pos+29]).to_string();
                println!("[ACTION] Snipping Code: {}", code);
                tokio::spawn(redeem(code, token.to_string(), start));
            }
        }
    }
}

async fn redeem(code: String, token: String, start: Instant) {
    let client = reqwest::Client::builder().use_rustls_tls().build().unwrap();
    let url = format!("https://discord.com/api/v9/entitlements/gift-codes/{}/redeem", code);
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    if let Ok(r) = client.post(url).headers(headers).send().await {
        println!("[RESULT] Status: {} | Time: {:?}", r.status(), start.elapsed());
    }
                         }
              
