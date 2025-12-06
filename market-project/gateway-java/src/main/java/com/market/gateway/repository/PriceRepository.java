package com.market.gateway.repository;

import com.market.gateway.model.Price;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.stereotype.Repository;
import java.util.List;

@Repository
public interface PriceRepository extends JpaRepository<Price, Long> {
    // Metoda veche (poate rămâne)
    List<Price> findTop50ByOrderByTimestampDesc();

    // NOU: Metoda pentru a lua istoricul doar pentru moneda selectată
    List<Price> findTop50BySymbolOrderByTimestampDesc(String symbol);
}