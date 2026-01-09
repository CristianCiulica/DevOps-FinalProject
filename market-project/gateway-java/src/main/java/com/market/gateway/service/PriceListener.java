package com.market.gateway.service;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.market.gateway.model.Alert;
import com.market.gateway.model.Price;
import com.market.gateway.repository.AlertRepository;
import com.market.gateway.repository.PriceRepository;
import org.springframework.amqp.rabbit.annotation.RabbitListener;
import org.springframework.messaging.simp.SimpMessagingTemplate;
import org.springframework.stereotype.Service;

@Service
public class PriceListener {

    private final PriceRepository priceRepository;
    private final AlertRepository alertRepository;
    private final SimpMessagingTemplate messagingTemplate;
    private final ObjectMapper objectMapper;
    public PriceListener(PriceRepository priceRepository,
                         AlertRepository alertRepository,
                         SimpMessagingTemplate messagingTemplate,
                         ObjectMapper objectMapper) {
        this.priceRepository = priceRepository;
        this.alertRepository = alertRepository;
        this.messagingTemplate = messagingTemplate;
        this.objectMapper = objectMapper;
    }

    @RabbitListener(queues = "market_prices")
    public void receivePrice(String message) {
        try {
            Price price = objectMapper.readValue(message, Price.class);
            priceRepository.save(price);
            if (Boolean.TRUE.equals(price.getIsAnomaly())) {
                Alert alert = new Alert();
                alert.setSymbol(price.getSymbol());
                alert.setTriggeredPrice(price.getPrice());
                alert.setMessage("RABBITMQ ALERT: High Volatility on " + price.getSymbol());
                alertRepository.save(alert);
                System.out.println("⚡ [RabbitMQ] Alert Saved: " + price.getSymbol());
            }

            messagingTemplate.convertAndSend("/topic/prices", price);

        } catch (Exception e) {
            System.err.println("❌ Error processing RabbitMQ message: " + e.getMessage());
            e.printStackTrace();
        }
    }
}