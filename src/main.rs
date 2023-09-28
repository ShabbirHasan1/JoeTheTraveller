use std::time::Duration;
use tokio::time;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
use serde_json::Value;

#[tokio::main]
async fn main() {
    let symbol = "BTCUSDT";

    loop {
        if let Err(err) = process_symbol_prices(symbol).await {
            eprintln!("Hata: {}", err);
        }

        time::sleep(Duration::from_secs(5)).await;
    }
}

async fn process_symbol_prices(symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
    let prices = get_prices(symbol).await?;
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&prices) {
        if let Some(ask_price) = json_value["askPrice"].as_str() {
            println!("{} ask price: {}", symbol, ask_price);
        } else {
            eprintln!("no ask price");
        }

        if let Some(bid_price) = json_value["bidPrice"].as_str() {
            println!("{} bid price: {}", symbol, bid_price);
        } else {
            eprintln!("no bid price");
        }
    } else {
        eprintln!("JSON error");
    }

    Ok(())
}

async fn get_prices(symbol: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://api.binance.com/api/v3/ticker/bookTicker?symbol={}", symbol);
    let client = reqwest::Client::new();
    let body = client.get(&url)
        .send()
        .await?
        .text()
        .await?;

    Ok(body)
}

fn calculate_rsi_for_symbol(symbol: &str, interval: &str) -> Option<f64> {
    let url = format!("https://api.binance.com/api/v1/klines?symbol={}&interval={}", symbol, interval);

    let response = reqwest::blocking::get(&url).ok()?.text().ok()?;
    let json_data: Value = serde_json::from_str(&response).ok()?;

    let mut closing_prices: Vec<f64> = Vec::new();

    for candle in json_data.as_array()? {
        let close_price = candle[4].as_str()?.parse::<f64>().ok()?;
        closing_prices.push(close_price);
    }

    Some(calculate_rsi(&closing_prices, 14))
}

fn calculate_rsi(closing_prices: &[f64], period: usize) -> f64 {
    let mut gains: Vec<f64> = Vec::new();
    let mut losses: Vec<f64> = Vec::new();

    for i in 1..closing_prices.len() {
        let price_diff = closing_prices[i] - closing_prices[i - 1];
        if price_diff >= 0.0 {
            gains.push(price_diff);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(price_diff.abs());
        }
    }

    let avg_gain: f64 = gains.iter().take(period).sum::<f64>() / period as f64;
    let avg_loss: f64 = losses.iter().take(period).sum::<f64>() / period as f64;

    let rs = avg_gain / avg_loss;
    let rsi = 100.0 - (100.0 / (1.0 + rs));

    rsi
}