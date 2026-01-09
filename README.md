# Crypto Market Data Aggregator & Trading Dashboard

![Java](https://img.shields.io/badge/Java-17-007396?style=flat-square&logo=java&logoColor=white)
![Spring Boot](https://img.shields.io/badge/Spring_Boot-3.2-6DB33F?style=flat-square&logo=spring&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-1.7-000000?style=flat-square&logo=rust&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15-4169E1?style=flat-square&logo=postgresql&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-Compose-2496ED?style=flat-square&logo=docker&logoColor=white)

## Project Overview

This project is a cloud-native, polyglot distributed system designed for real-time aggregation, analysis, and visualization of cryptocurrency market data. It demonstrates a microservices architecture where high-performance data processing is handled by **Rust**, business logic and security by **Java Spring Boot**, and persistence by **PostgreSQL**.

The system simulates a high-frequency trading environment where market data is ingested, processed for anomalies and technical indicators (Moving Averages), and broadcasted to a web client via **WebSockets** with low latency.

---

## System Architecture

The solution consists of three main containerized services orchestrated via Docker Compose:

1.  **Aggregator Service (Rust)**
    * Acts as the data ingestion engine.
    * Fetches live market data from external APIs (Binance).
    * **Data Processing:** Maintains an in-memory sliding window to calculate a 5-point Moving Average (MA) for each asset.
    * **Anomaly Detection:** Algorithmic detection of price spikes (>5% deviation from the MA).
    * Transmits processed payloads (Price + MA + Anomaly Flags) to the Gateway via HTTP.

2.  **Gateway Service (Java Spring Boot)**
    * **API Layer:** REST API for historical data retrieval and AI analysis.
    * **Security:** Implements full authentication and authorization using Spring Security backed by PostgreSQL (BCrypt password hashing).
    * **Real-time Broker:** Uses STOMP over WebSockets to broadcast updates to connected clients immediately upon ingestion.
    * **Persistence:** Stores raw price data, calculated metrics, and detected alerts in the database.

3.  **Database (PostgreSQL)**
    * Relational storage for:
        * `users`: Account credentials and roles.
        * `prices`: Time-series data for assets.
        * `alerts`: Audit log of detected market anomalies.

---

## Key Features

### 1. Data Aggregation & Analysis
Unlike simple proxy applications, the Rust microservice performs edge computing:
* **Moving Average Calculation:** Computes the trend line dynamically before data reaches the database.
* **Statistical Anomaly Detection:** Filters noise and flags significant market events automatically.

### 2. Security & Authentication
* **Database-backed Authentication:** Not limited to in-memory users.
* **Registration Flow:** Fully functional user registration system (`/register.html`) with duplicate user checks.
* **Session Management:** Secure login/logout flows with encrypted passwords.

### 3. Real-Time Visualization
* **Dual-Line Charting:** Visualizes both the raw price (Blue) and the Rust-calculated Moving Average (Orange/Dotted) simultaneously.
* **Live Updates:** The frontend updates via WebSocket push notifications, eliminating the need for page refreshing.

### 4. AI Market Sentiment
* Integration with **Llama 3 (via Groq API)** to provide contextual market analysis.
* The system uses a custom prompt engineering strategy to generate professional, "late-cycle bull market" technical analysis summaries.

---

## Technical Stack

* **Backend:** Java 17, Spring Boot 3.2, Spring Security, Spring Data JPA, Lombok.
* **Aggregator:** Rust (edition 2021), Reqwest (HTTP Client), Serde (Serialization).
* **Database:** PostgreSQL 15.
* **Frontend:** HTML5, Bootstrap 5, Chart.js, SockJS, STOMP.
* **DevOps:** Docker, Docker Compose, GitHub Actions (CI/CD).

---

## Getting Started

### Prerequisites
* Docker Desktop (Engine 20.10+)
* Docker Compose

### Installation & Running

1.  **Clone the repository:**
    ```bash
    git clone <repository-url>
    cd market-project
    ```

2.  **Clean previous volumes (Recommended):**
    This ensures the database schema is initialized correctly with the new user tables.
    ```bash
    docker compose down -v
    ```

3.  **Build and Start:**
    This command compiles the Java JAR and the Rust binary inside their respective containers.
    ```bash
    docker compose up --build
    ```

4.  **Access the Application:**
    * Navigate to: `http://localhost:8080`
    * You will be redirected to the Login page.

### Default Credentials
The application includes a `DataInitializer` that seeds a demo account upon startup:
* **Username:** `student`
* **Password:** `student`

Alternatively, you can create a new account using the **"Create one"** link on the login page.

---

## API Documentation

The Java Gateway exposes the following REST endpoints:

### Public / Ingestion
* `POST /api/ingest`
    * Used by the Rust Aggregator to send processed data.
    * Payload: `{ "symbol": "BTC-USD", "price": 90000.0, "averagePrice": 89500.0, "isAnomaly": false }`

### Protected (Requires Auth)
* `GET /api/prices?symbol={symbol}`
    * Returns the last 50 data points for a specific asset (reversed chronological order).
* `GET /api/ai-analysis?symbol={symbol}`
    * Triggers the AI service to generate a text-based analysis of the asset.
* `POST /perform_register`
    * Handles new user creation.

---

## CI/CD Pipeline

The project includes a GitHub Actions workflow (`maven.yml`) that validates the integrity of the codebase.
* **Build:** Compiles the Java application.
* **Test:** Runs unit tests (if applicable).
* **Containerization:** Verifies that the Docker image can be built successfully.

---

## Database Schema

The PostgreSQL database initializes with the following structure:

* **Table `users`**: `id`, `username`, `password` (hashed), `role`.
* **Table `prices`**: `id`, `symbol`, `price`, `average_price`, `is_anomaly`, `timestamp`.
* **Table `alerts`**: `id`, `symbol`, `triggered_price`, `message`, `timestamp`.

---

## License

This project is developed for educational purposes as part of a Distributed Systems course.
