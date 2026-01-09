package com.market.gateway.controller;

import com.market.gateway.model.Alert;
import com.market.gateway.model.Price;
import com.market.gateway.repository.AlertRepository;
import com.market.gateway.repository.PriceRepository;
import com.market.gateway.service.GeminiService;
import org.springframework.http.ResponseEntity;
import org.springframework.messaging.simp.SimpMessagingTemplate;
import org.springframework.web.bind.annotation.*;

import java.util.List;

@RestController
@RequestMapping("/api")
@CrossOrigin(origins = "*")
public class PriceController {

    private final PriceRepository priceRepository;
    private final AlertRepository alertRepository;
    private final SimpMessagingTemplate messagingTemplate;
    private final GeminiService geminiService;

    public PriceController(PriceRepository priceRepository,
                           AlertRepository alertRepository,
                           SimpMessagingTemplate messagingTemplate,
                           GeminiService geminiService) {
        this.priceRepository = priceRepository;
        this.alertRepository = alertRepository;
        this.messagingTemplate = messagingTemplate;
        this.geminiService = geminiService;
    }

    @GetMapping("/ai-analysis")
    public ResponseEntity<String> getAiAnalysis(@RequestParam(defaultValue = "BTC-USD") String symbol) {
        String analysis = geminiService.getMarketPrediction(symbol);
        return ResponseEntity.ok(analysis);
    }

    @PostMapping("/ingest")
    public ResponseEntity<String> ingestPrice(@RequestBody Price price) {
        priceRepository.save(price);

        if (Boolean.TRUE.equals(price.getIsAnomaly())) {
            Alert alert = new Alert();
            alert.setSymbol(price.getSymbol());
            alert.setTriggeredPrice(price.getPrice());
            alert.setMessage("ANOMALY DETECTED: Price deviation on " + price.getSymbol());
            alertRepository.save(alert);

            System.out.println("!!! ALERT SAVED for " + price.getSymbol());
        }

        messagingTemplate.convertAndSend("/topic/prices", price);
        return ResponseEntity.ok("Data received");
    }

    @GetMapping("/prices")
    public List<Price> getAllPrices(@RequestParam(required = false) String symbol) {
        if (symbol != null && !symbol.isEmpty()) {
            return priceRepository.findTop50BySymbolOrderByTimestampDesc(symbol);
        }
        return priceRepository.findTop50ByOrderByTimestampDesc();
    }
}