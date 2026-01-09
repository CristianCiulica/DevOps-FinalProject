use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;

#[derive(Deserialize, Debug)]
struct BinanceTicker {
    symbol: String,
    price: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PriceData {
    symbol: String,
    price: f64,
    average_price: f64,
    source: String,
    timestamp: u64,
    is_anomaly: bool,
}

fn main() {
    let gateway_url = std::env::var("GATEWAY_URL").unwrap_or("http://java-gateway:8080/api/ingest".to_string());
    let client = Client::new();
    let mut rng = rand::thread_rng();
    let symbols = vec![
        ("BTCUSDT", "BTC-USD"),
        ("ETHUSDT", "ETH-USD"),
        ("SOLUSDT", "SOL-USD"),
        ("ADAUSDT", "ADA-USD")
    ];

    let mut price_history: HashMap<String, Vec<f64>> = HashMap::new();

    println!("Rust Aggregator v2.0 LIVE: Monitoring with Moving Average & Anomaly Detection");
    loop {
        let start_cycle = Instant::now();
        let mut processed_count = 0;

        for (binance_symbol, display_symbol) in &symbols {
            let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", binance_symbol);
            let current_price: f64;
            let source_label: String;

            match client.get(&url).timeout(Duration::from_secs(2)).send() {
                Ok(resp) => {
                    if let Ok(ticker) = resp.json::<BinanceTicker>() {
                        current_price = ticker.price.parse().unwrap_or(0.0);
                        source_label = String::from("Binance-API");
                    } else {
                        let base = if *display_symbol == "BTC-USD" { 90000.0 } else { 100.0 };
                        current_price = rng.gen_range(base..base*1.05);
                        source_label = String::from("Backup-Gen-Error");
                    }
                },
                Err(_) => {
                    let base = if *display_symbol == "BTC-USD" { 90000.0 } else { 100.0 };
                    current_price = rng.gen_range(base..base*1.05);
                    source_label = String::from("Backup-Gen-Net");
                }
            }

            let history = price_history.entry(display_symbol.to_string()).or_insert(Vec::new());
            history.push(current_price);

            if history.len() > 5 {
                history.remove(0);
            }

            let avg_price: f64 = history.iter().sum::<f64>() / history.len() as f64;

            let deviation = (current_price - avg_price).abs() / avg_price;
            let is_spike = deviation > 0.05;

            let anomaly = is_spike || (display_symbol == &"BTC-USD" && (current_price > 99000.0 || current_price < 80000.0));

            if anomaly {
                println!("⚠️ ALARM: Anomaly detected for {}! Price: {}, Avg: {}", display_symbol, current_price, avg_price);
            }

            let price_packet = PriceData {
                symbol: display_symbol.to_string(),
                price: current_price,
                average_price: avg_price,
                source: source_label,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                is_anomaly: anomaly,
            };

            if let Err(e) = client.post(&gateway_url).json(&price_packet).send() {
                eprintln!("Failed sending {}: {}", display_symbol, e);
            }

            processed_count += 1;
            thread::sleep(Duration::from_millis(200));
        }

        let duration = start_cycle.elapsed();
        println!("METRIC [Latency]: Processed {} symbols in {:?}. Aggregation active.", processed_count, duration);

        thread::sleep(Duration::from_secs(3));
    }
}