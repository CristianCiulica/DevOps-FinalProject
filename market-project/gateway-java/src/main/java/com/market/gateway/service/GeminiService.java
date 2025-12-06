package com.market.gateway.service;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;

@Service
public class GeminiService {

    @Value("${groq.api.key}")
    private String apiKey;

    private final String GROQ_URL = "https://api.groq.com/openai/v1/chat/completions";
    private final String FEAR_GREED_API = "https://api.alternative.me/fng/?limit=1";

    public String getMarketPrediction(String symbol) {
        try {
            // 1. Luăm sentimentul pieței (Fear/Greed) ca să pară că știe despre ce vorbește
            String fearAndGreedData = fetchFearAndGreed();

            // 2. PROMPT STRICT: Fără prețuri, doar HYPE
            String promptText = "You are a hyper-energetic Crypto YouTuber shilling " + symbol + ". " +
                    "The market sentiment is " + fearAndGreedData + ". " +
                    "Don't mention specific prices. Just give a super bullish, exciting 2-sentence update. " +
                    "Use slang like 'LFG', 'To the Moon', 'Rocket', '100x soon'. " +
                    "Tell the viewers why " + symbol + " is the best investment right now!";

            return callGroq(promptText);

        } catch (Exception e) {
            return "Hype Error: " + e.getMessage();
        }
    }

    private String callGroq(String text) throws Exception {
        String jsonBody = "{"
                + "\"model\": \"llama-3.3-70b-versatile\","
                + "\"messages\": [{\"role\": \"user\", \"content\": \"" + text + "\"}],"
                + "\"temperature\": 0.9" // Temperature mai mare = mai creativ/nebun
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
            String result = body.substring(start, end);
            // Curățăm caracterele speciale JSON
            return result.replace("\\n", " ").replace("\\\"", "\"");
        }

        return "Groq API Error";
    }

    private String fetchFearAndGreed() {
        try {
            HttpClient client = HttpClient.newHttpClient();
            HttpRequest request = HttpRequest.newBuilder().uri(URI.create(FEAR_GREED_API)).GET().build();
            HttpResponse<String> response = client.send(request, HttpResponse.BodyHandlers.ofString());
            String body = response.body();

            if(body.contains("\"value\":")) {
                int classIndex = body.indexOf("\"value_classification\":") + 24;
                String classification = body.substring(classIndex, body.indexOf("\"", classIndex));
                return classification;
            }
            return "Neutral";
        } catch (Exception e) {
            return "Unknown";
        }
    }
}