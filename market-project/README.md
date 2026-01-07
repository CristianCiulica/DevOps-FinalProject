# Market Aggregator Project – Documentation

This documentation describes the project's current functionality (based on the code in the repository), including services, endpoints, WebSocket implementation, Docker Compose orchestration, Postgres persistence, and observability.

—

## Quick Summary

- Microservices:
  - Java Gateway (Spring Boot): REST + WebSocket + JPA.
  - Rust Aggregator: Collects prices (Binance or fallback) and sends them to the gateway.
  - Postgres: Data storage.
- Frontend: Simple static `index.html` file, live display via WebSocket (STOMP).
- Deployment: Docker Compose.

—

## How to Run

### Prerequisites

- Docker Desktop
- Optional: Java 17, Maven, Rust (for local development)

### Relevant Files

- `docker-compose.yml`
- `gateway-java/Dockerfile`, `aggregator-rust/Dockerfile`
- `.env` (Contains variables: `DB_PASSWORD`, `GROQ_API_KEY`)

### Steps

```powershell
# Start all services (build + run)
cd market-project
docker compose --env-file .env up --build -d

# Check status
docker compose ps

# Logs (useful for debugging)
docker compose logs -f java-gateway ; docker compose logs -f rust-aggregator ; docker compose logs -f market-db

# Stop + clean DB volumes (Warning: deletes data)
docker compose down -v
```

### Access

- REST API Gateway: http://localhost:8080
- WebSocket STOMP endpoint: ws://localhost:8080/ws-market (Topic: `/topic/prices`)
- Dashboard: http://localhost:8080/index.html
- Postgres: Mapped port 5432 (Container `market-db`)

—

## Current Architecture

1. **Gateway (Java Spring Boot)**
   - **Controller:** `PriceController`
     - `POST /api/ingest`: Receives prices from the aggregator and saves them; broadcasts via WebSocket to `/topic/prices`.
     - `GET /api/prices`: Returns the last 50 prices; supports optional filtering `?symbol=...`.
     - `GET /api/ai-analysis`: Calls `GeminiService` for text analysis (uses `GROQ_API_KEY`).
   - **WebSocket:** `WebSocketConfig`
     - STOMP endpoint: `/ws-market` (SockJS enabled).
     - Simple broker: `/topic`.
   - **Persistence:** Price entity (table `prices`) with fields:
     - `id`, `symbol`, `price`, `averagePrice`, `isAnomaly`, `timestamp`.
     - `timestamp` is automatically set on insert if missing.
   - **Repository:** `PriceRepository`
     - `findTop50ByOrderByTimestampDesc()`
     - `findTop50BySymbolOrderByTimestampDesc(symbol)`
   - **App Config:** `application.properties`
     - `server.port=8080`
     - `spring.jpa.hibernate.ddl-auto=update`
     - Springdoc (Swagger UI) active: `/swagger-ui.html`, docs: `/api-docs`.

2. **Aggregator (Rust)**
   - `main.rs`: Continuous loop over symbols: BTCUSDT, ETHUSDT, SOLUSDT, ADAUSDT.
   - Attempts to read current price from `https://api.binance.com/api/v3/ticker/price?symbol=...`.
     - On failure, generates fallback prices (random) around base values.
   - **Simple Anomaly Detection:**
     - For BTC-USD: `price > 99000.0 || price < 80000.0`.
     - For other symbols: `false`.
   - **JSON Construction:** `{ symbol, price, source, timestamp, is_anomaly }`.
   - **Transmission:** Sends via HTTP POST to Gateway: `GATEWAY_URL` (default: `http://java-gateway:8080/api/ingest`).
   - **Interval:** ~500ms between symbols and ~3s at the end of the cycle.

3. **Postgres (Database)**
   - Image: `postgres:15-alpine`.
   - Variables: `POSTGRES_DB=market_data`, `POSTGRES_USER=student`, `POSTGRES_PASSWORD` from `.env`.
   - Healthcheck: `pg_isready`.

4. **Orchestration (`docker-compose.yml`)**
   - Services: `db`, `gateway`, `aggregator`.
   - Network: `market-net`.
   - `gateway` depends on `db` (service_healthy).
   - `aggregator` depends on `gateway`.

—

## Exposed Endpoints (Current Implementation)

- `POST /api/ingest`
  - JSON Body (accepted by `PriceController` via `Price` entity):
    - Minimum valid example: `{ "symbol": "BTC-USD", "price": 91000.0, "averagePrice": 0, "isAnomaly": false, "timestamp": "2024-01-01T00:00:00Z" }`
    - Note: Aggregator sends `timestamp` as UNIX seconds (number), but `Price.timestamp` is `ZonedDateTime`. Hibernate attempts mapping. If errors occur, adapt the timestamp format in the aggregator or the DTO.
  - Effect: Saves to DB, broadcasts via WebSocket to `/topic/prices`.

- `GET /api/prices`
  - Optional Parameter: `symbol`.
  - Returns: List (max 50) ordered descending by `timestamp`.

- `GET /api/ai-analysis?symbol=BTC-USD`
  - Returns: A string (result from `GeminiService`) – depends on `GROQ_API_KEY`.

**WebSocket:**

- Endpoint: `/ws-market` (STOMP with SockJS).
- Broadcast Topic: `/topic/prices`.

—

## Frontend (Dashboard)

- File: `gateway-java/src/main/resources/static/index.html`.
- Uses STOMP/SockJS at endpoint `/ws-market` and listens on `/topic/prices`.
- Displays live prices and highlights the `isAnomaly` field.
- Simple interface, suitable for real-time demonstration.

—

## Observability (Current State)

- Spring Boot Actuator is listed in `pom.xml`, but no additional configuration exists in `application.properties`; the base endpoint `/actuator/health` should be available.
- No custom metrics (latency, throughput) exposed in Java or Rust code.
- Logs are default (not structured JSON).

—

## Docker Compose – Details and Observations

- Gateway receives the following env vars in compose:
  - `SPRING_DATASOURCE_URL=jdbc:postgresql://db:5432/market_data`
  - `SPRING_DATASOURCE_USERNAME=student`
  - `SPRING_DATASOURCE_PASSWORD=${DB_PASSWORD}`
  - `SPRING_JPA_HIBERNATE_DDL_AUTO=update`
  - `GROQ_API_KEY=${GROQ_API_KEY}`
- `application.properties` has `spring.datasource.url=jdbc:postgresql://postgres:5432/market_data` and user/pass from `${POSTGRES_USER}`/`${POSTGRES_PASSWORD}` – this may conflict with what compose sends (host `db` vs `postgres`). Inside containers, the service is named `db`.

—

## Technologies Used

- **Gateway:** Spring Boot 3.2, Web, WebSocket (STOMP + SockJS), Spring Data JPA, PostgreSQL, Springdoc OpenAPI, Actuator, Lombok.
- **Aggregator:** Rust, `reqwest` (blocking client), `serde`, `rand`.
- **DB:** Postgres 15.
- **Orchestration:** Docker Compose
- **Frontend:** HTML, JavaScript, CSS

—
## Future Improvements
- Add structured logging (JSON) for better log management.
- Implement custom metrics (latency, throughput) in both services.
- Enhance anomaly detection logic in the Rust aggregator.
- Improve frontend UI for better user experience.
- Add unit and integration tests for services.
- Implement error handling and retries in the Rust aggregator for network failures.
- Secure WebSocket connections (WSS) for production environments.
- Add authentication and authorization for REST endpoints.
- Implement database migrations (e.g., Flyway or Liquibase) for better schema management.

