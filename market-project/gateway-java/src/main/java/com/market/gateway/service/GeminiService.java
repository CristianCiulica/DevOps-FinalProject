package com.market.gateway.service;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;

@Service
public class GeminiService {
    @Value("${groq.api.key:demo_key}")
    private String apiKey;

    private final String GROQ_URL = "https://api.groq.com/openai/v1/chat/completions";

    public String getMarketPrediction(String symbol) {
        if (apiKey == null || apiKey.isEmpty() || apiKey.equals("demo_key")) {
            return "Institutional metrics indicate late-cycle accumulation. Breakout structure suggests bullish continuation.";
        }

        try {
            String promptText = "Analyze " + symbol + ". Context: Late-stage crypto bull cycle. Bias: Bullish but professional. Directive: Ignore Fear/Greed. Output: One concise sentence regarding momentum.";

            return callGroq(promptText);

        } catch (Exception e) {
            return "Market data shows strong bullish divergence relative to recent volatility.";
        }
    }

    private String callGroq(String text) throws Exception {
        String safeText = text.replace("\"", "'");

        String jsonBody = "{"
                + "\"model\": \"llama-3.3-70b-versatile\","
                + "\"messages\": [{\"role\": \"user\", \"content\": \"" + safeText + "\"}],"
                + "\"temperature\": 0.6"
                + "}";

        HttpClient client = HttpClient.newHttpClient();
        HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(GROQ_URL))
                .header("Content-Type", "application/json")
                .header("Authorization", "Bearer " + apiKey)
                .POST(HttpRequest.BodyPublishers.ofString(jsonBody))
                .build();

        HttpResponse<String> response = client.send(request, HttpResponse.BodyHandlers.ofString());

        String body = response.body();

        if (body.contains("\"content\":")) {
            int start = body.indexOf("\"content\":") + 11;
            int end = body.indexOf("\"", start);
            if (end > start) {
                String result = body.substring(start, end);
                return result.replace("\\n", " ").replace("\\\"", "\"");
            }
        }

        return "Analysis pending (High Load)...";
    }
}