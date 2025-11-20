package com.market.gateway.controller;

import com.market.gateway.model.Price;
import com.market.gateway.repository.PriceRepository;
import org.springframework.http.ResponseEntity;
import org.springframework.messaging.simp.SimpMessagingTemplate; // <--- Import Nou
import org.springframework.web.bind.annotation.*;

import java.util.List;

@RestController
@RequestMapping("/api")
@CrossOrigin(origins = "*")
public class PriceController {

    private final PriceRepository priceRepository;
    private final SimpMessagingTemplate messagingTemplate; // <--- Unealta de WebSocket

    public PriceController(PriceRepository priceRepository, SimpMessagingTemplate messagingTemplate) {
        this.priceRepository = priceRepository;
        this.messagingTemplate = messagingTemplate;
    }

    @PostMapping("/ingest")
    public ResponseEntity<String> ingestPrice(@RequestBody Price price) {
        // 1. Salvăm în bază
        priceRepository.save(price);

        // 2. Trimitem LIVE către Frontend
        messagingTemplate.convertAndSend("/topic/prices", price);

        System.out.println("📥 Primit & Trimis Live: " + price.getPrice());

        return ResponseEntity.ok("Data received");
    }

    @GetMapping("/prices")
    public List<Price> getAllPrices() {
        return priceRepository.findTop50ByOrderByTimestampDesc();
    }
}