use std::thread;
use std::time::Duration;
use rand::Rng;
use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;

#[derive(Deserialize, Debug)]
struct BinanceTicker {
    symbol: String,
    price: String,
}

#[derive(Serialize)]
struct PriceData {
    symbol: String,
    price: f64,
    source: String,
    timestamp: u64,
    is_anomaly: bool,
}

fn main() {
    let gateway_url = std::env::var("GATEWAY_URL").unwrap_or("http://java-gateway:8080/api/ingest".to_string());
    let client = Client::new();
    let mut rng = rand::thread_rng();

    // Lista de monede pe care le urmărim (Binance Symbols)
    let symbols = vec![
        ("BTCUSDT", "BTC-USD"),
        ("ETHUSDT", "ETH-USD"),
        ("SOLUSDT", "SOL-USD"),
        ("ADAUSDT", "ADA-USD")
    ];

    println!("Rust Aggregator LIVE: Monitoring {:?}", symbols);

    loop {
        for (binance_symbol, display_symbol) in &symbols {
            let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", binance_symbol);
            let current_price: f64;
            let source_label: String;

            match client.get(&url).send() {
                Ok(resp) => {
                    if let Ok(ticker) = resp.json::<BinanceTicker>() {
                        current_price = ticker.price.parse().unwrap_or(0.0);
                        source_label = String::from("Binance-API");
                    } else {
                        // Fallback random simulat diferit pt fiecare
                        let base = if *display_symbol == "BTC-USD" { 90000.0 } else { 3000.0 };
                        current_price = rng.gen_range(base..base+100.0);
                        source_label = String::from("Backup-Gen");
                    }
                },
                Err(_) => {
                    let base = if *display_symbol == "BTC-USD" { 90000.0 } else { 3000.0 };
                    current_price = rng.gen_range(base..base+100.0);
                    source_label = String::from("Backup-Gen");
                }
            }

            // Anomalie simpla (logica poate fi complexa per moneda)
            let anomaly = if *display_symbol == "BTC-USD" {
                current_price > 99000.0 || current_price < 80000.0
            } else {
                false // Simplificat pentru restul
            };

            let price_packet = PriceData {
                symbol: display_symbol.to_string(),
                price: current_price,
                source: source_label,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                is_anomaly: anomaly,
            };

            // Trimitem datele
            if let Err(e) = client.post(&gateway_url).json(&price_packet).send() {
                eprintln!("❌ Failed sending {}: {}", display_symbol, e);
            }

            // Mica pauza intre request-uri ca sa nu luam rate limit
            thread::sleep(Duration::from_millis(500));
        }

        // Pauza dupa ce am trecut prin toate monedele
        thread::sleep(Duration::from_secs(3));
    }
}