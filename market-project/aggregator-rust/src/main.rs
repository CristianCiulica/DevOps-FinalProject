use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rand::Rng;
use serde::{Deserialize, Serialize};
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties};
use tokio::time;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Rust Aggregator v3.1 (Event Driven + Durable) LIVE: Connecting to RabbitMQ...");

    // 1. Conectare la RabbitMQ
    let addr = std::env::var("RABBITMQ_ADDR").unwrap_or_else(|_| "amqp://guest:guest@rabbitmq:5672/%2f".into());

    // Retry logic simplu pentru conectare (asteptam sa porneasca containerul)
    let connection = loop {
        match Connection::connect(&addr, ConnectionProperties::default()).await {
            Ok(conn) => break conn,
            Err(_) => {
                println!("Waiting for RabbitMQ...");
                time::sleep(Duration::from_secs(2)).await;
            }
        }
    };

    let channel = connection.create_channel().await?;

    // 2. Declaram coada cu DURABLE: TRUE (Fix-ul critic pentru a fi compatibil cu Java)
    let _queue = channel
        .queue_declare(
            "market_prices",
            QueueDeclareOptions {
                durable: true, // <--- Aici a fost problema, acum e setat corect pe true
                ..QueueDeclareOptions::default()
            },
            FieldTable::default(),
        )
        .await?;

    println!("✅ Connected to RabbitMQ! Starting Stream...");

    let client = reqwest::Client::new();
    let mut rng = rand::thread_rng();

    let symbols = vec![
        ("BTCUSDT", "BTC-USD"),
        ("ETHUSDT", "ETH-USD"),
        ("SOLUSDT", "SOL-USD"),
        ("ADAUSDT", "ADA-USD")
    ];

    let mut price_history: HashMap<String, Vec<f64>> = HashMap::new();

    loop {
        let start_cycle = std::time::Instant::now();
        let mut processed_count = 0;

        for (binance_symbol, display_symbol) in &symbols {
            let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", binance_symbol);
            let current_price: f64;
            let source_label: String;

            match client.get(&url).timeout(Duration::from_secs(2)).send().await {
                Ok(resp) => {
                    if let Ok(ticker) = resp.json::<BinanceTicker>().await {
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
            if history.len() > 5 { history.remove(0); }
            let avg_price: f64 = history.iter().sum::<f64>() / history.len() as f64;

            let deviation = (current_price - avg_price).abs() / avg_price;
            let is_spike = deviation > 0.05;
            let anomaly = is_spike || (display_symbol == &"BTC-USD" && (current_price > 99000.0 || current_price < 80000.0));

            if anomaly {
                println!("⚠️ ANOMALY: {} Price: {}", display_symbol, current_price);
            }

            let price_packet = PriceData {
                symbol: display_symbol.to_string(),
                price: current_price,
                average_price: avg_price,
                source: source_label,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                is_anomaly: anomaly,
            };

            let payload = serde_json::to_vec(&price_packet).unwrap();

            channel
                .basic_publish(
                    "",
                    "market_prices",
                    BasicPublishOptions::default(),
                    &payload,
                    BasicProperties::default(),
                )
                .await
                .expect("Failed to publish");

            processed_count += 1;
            time::sleep(Duration::from_millis(200)).await;
        }

        let duration = start_cycle.elapsed();
        println!("METRIC [Latency]: Processed {} symbols in {:?} via RabbitMQ.", processed_count, duration);
        time::sleep(Duration::from_secs(3)).await;
    }
}