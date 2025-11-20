use std::thread;
use std::time::Duration;
use rand::Rng;
use serde::Serialize;
use reqwest::blocking::Client;

#[derive(Serialize)]
struct PriceData {
    symbol: String,
    price: f64,
    source: String,
    timestamp: u64,
    is_anomaly: bool,
}

fn main() {
    // URL-ul corect către Java (inclusiv /api/)
    let gateway_url = "http://java-gateway:8080/api/ingest";
    let client = Client::new();
    let mut rng = rand::thread_rng();

    println!("🚀 Rust Aggregator pornit! Generez preturi BTC (reqwest)...");

    loop {
        // 1. Generăm un preț BTC realist (între 90k și 92k)
        let current_price: f64 = rng.gen_range(90000.0..92000.0);

        // 2. Definim anomalia (peste 91.800)
        let anomaly = current_price > 91800.0;

        // 3. Creăm pachetul JSON
        // Important: Numele câmpurilor trebuie să se potrivească cu clasa Java 'Price'
        // Java: symbol, price, isAnomaly (Rust le convertește automat dacă folosim serde)
        let price_packet = PriceData {
            symbol: String::from("BTC-USD"),
            price: (current_price * 100.0).round() / 100.0,
            source: String::from("Rust-Generator"),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_anomaly: anomaly,
        };

        // 4. Trimitem către Java
        match client.post(gateway_url).json(&price_packet).send() {
            Ok(_) => {
                if anomaly {
                    println!("⚠️ ANOMALIE trimisă: ${:.2}", current_price);
                } else {
                    println!("✅ Preț trimis: ${:.2}", current_price);
                }
            }
            Err(e) => eprintln!("❌ Eroare conexiune Java: {}", e),
        }

        // 5. Așteptăm
        thread::sleep(Duration::from_secs(1));
    }
}