//! Pricing Service calculation tests
//! Tests for price estimation and adjustments

#[cfg(test)]
mod tests {
    // Market analysis structure
    #[derive(Debug, Clone)]
    struct MarketData {
        brand: String,
        model: String,
        average_price: f64,
        market_demand: f64, // 0.5 to 2.0 (low to high)
        seasonal_adjustment: f64, // 0.9 to 1.1
    }

    // Calculate market-based price
    fn calculate_market_price(market_data: &MarketData, base_valuation: f64) -> f64 {
        let demand_factor = market_data.market_demand;
        let seasonal_factor = market_data.seasonal_adjustment;

        (base_valuation * demand_factor * seasonal_factor).round()
    }

    // Add profit margin
    fn apply_profit_margin(price: f64, margin_percent: f64) -> f64 {
        price * (1.0 + margin_percent / 100.0)
    }

    // Apply regional adjustment
    fn apply_regional_adjustment(price: f64, region: &str) -> f64 {
        let adjustment = match region.to_lowercase().as_str() {
            "north" => 1.05,
            "south" => 0.95,
            "east" => 1.02,
            "west" => 0.98,
            "central" => 1.0,
            _ => 1.0,
        };

        price * adjustment
    }

    // Calculate final price with all factors
    fn calculate_final_price(
        base_valuation: f64,
        market_data: &MarketData,
        margin_percent: f64,
        region: &str,
    ) -> f64 {
        let market_adjusted = calculate_market_price(market_data, base_valuation);
        let with_margin = apply_profit_margin(market_adjusted, margin_percent);
        let regional_adjusted = apply_regional_adjustment(with_margin, region);

        (regional_adjusted / 100.0).round() * 100.0 // Round to nearest 100
    }

    // ── Market Price Calculation Tests ─────────────────────────────────

    #[test]
    fn test_market_price_with_high_demand() {
        let market_data = MarketData {
            brand: "Tesla".to_string(),
            model: "Model 3".to_string(),
            average_price: 30000.0,
            market_demand: 1.5, // High demand
            seasonal_adjustment: 1.0,
        };

        let price = calculate_market_price(&market_data, 25000.0);
        // 25000 * 1.5 * 1.0 = 37500
        assert_eq!(price, 37500.0);
    }

    #[test]
    fn test_market_price_with_low_demand() {
        let market_data = MarketData {
            brand: "Old Brand".to_string(),
            model: "Old Model".to_string(),
            average_price: 5000.0,
            market_demand: 0.6, // Low demand
            seasonal_adjustment: 1.0,
        };

        let price = calculate_market_price(&market_data, 10000.0);
        // 10000 * 0.6 * 1.0 = 6000
        assert_eq!(price, 6000.0);
    }

    #[test]
    fn test_market_price_seasonal_adjustment() {
        let market_data = MarketData {
            brand: "Generic".to_string(),
            model: "Model".to_string(),
            average_price: 15000.0,
            market_demand: 1.0,
            seasonal_adjustment: 1.1, // High season
        };

        let price = calculate_market_price(&market_data, 10000.0);
        // 10000 * 1.0 * 1.1 = 11000
        assert_eq!(price, 11000.0);
    }

    #[test]
    fn test_market_price_low_season() {
        let market_data = MarketData {
            brand: "Generic".to_string(),
            model: "Model".to_string(),
            average_price: 15000.0,
            market_demand: 1.0,
            seasonal_adjustment: 0.9, // Low season
        };

        let price = calculate_market_price(&market_data, 10000.0);
        // 10000 * 1.0 * 0.9 = 9000
        assert_eq!(price, 9000.0);
    }

    // ── Profit Margin Tests ────────────────────────────────────────────

    #[test]
    fn test_apply_5_percent_margin() {
        let price = 10000.0;
        let result = apply_profit_margin(price, 5.0);

        assert_eq!(result, 10500.0);
    }

    #[test]
    fn test_apply_10_percent_margin() {
        let price = 10000.0;
        let result = apply_profit_margin(price, 10.0);

        assert_eq!(result, 11000.0);
    }

    #[test]
    fn test_apply_zero_margin() {
        let price = 10000.0;
        let result = apply_profit_margin(price, 0.0);

        assert_eq!(result, 10000.0);
    }

    #[test]
    fn test_apply_negative_margin_discount() {
        let price = 10000.0;
        let result = apply_profit_margin(price, -5.0);

        assert_eq!(result, 9500.0);
    }

    // ── Regional Adjustment Tests ──────────────────────────────────────

    #[test]
    fn test_north_region_5_percent_increase() {
        let price = 10000.0;
        let result = apply_regional_adjustment(price, "north");

        assert_eq!(result, 10500.0);
    }

    #[test]
    fn test_south_region_5_percent_decrease() {
        let price = 10000.0;
        let result = apply_regional_adjustment(price, "south");

        assert_eq!(result, 9500.0);
    }

    #[test]
    fn test_east_region_2_percent_increase() {
        let price = 10000.0;
        let result = apply_regional_adjustment(price, "east");

        assert_eq!(result, 10200.0);
    }

    #[test]
    fn test_central_region_no_adjustment() {
        let price = 10000.0;
        let result = apply_regional_adjustment(price, "central");

        assert_eq!(result, 10000.0);
    }

    #[test]
    fn test_unknown_region_no_adjustment() {
        let price = 10000.0;
        let result = apply_regional_adjustment(price, "unknown");

        assert_eq!(result, 10000.0);
    }

    #[test]
    fn test_region_case_insensitive() {
        let price = 10000.0;
        assert_eq!(apply_regional_adjustment(price, "NORTH"), 10500.0);
        assert_eq!(apply_regional_adjustment(price, "North"), 10500.0);
    }

    // ── Final Price Calculation Tests ──────────────────────────────────

    #[test]
    fn test_complete_price_calculation() {
        let market_data = MarketData {
            brand: "Toyota".to_string(),
            model: "Corolla".to_string(),
            average_price: 15000.0,
            market_demand: 1.0,
            seasonal_adjustment: 1.0,
        };

        let final_price = calculate_final_price(15000.0, &market_data, 8.0, "north");

        // Base: 15000
        // Market: 15000 * 1.0 * 1.0 = 15000
        // Margin (8%): 15000 * 1.08 = 16200
        // Regional (north +5%): 16200 * 1.05 = 17010
        // Rounded to 100: 17000
        assert_eq!(final_price, 17000.0);
    }

    #[test]
    fn test_luxury_vehicle_pricing() {
        let market_data = MarketData {
            brand: "BMW".to_string(),
            model: "M5".to_string(),
            average_price: 80000.0,
            market_demand: 1.3, // High demand
            seasonal_adjustment: 1.1, // High season
        };

        let final_price = calculate_final_price(70000.0, &market_data, 10.0, "central");

        // Base: 70000
        // Market: 70000 * 1.3 * 1.1 = 100100
        // Margin (10%): 100100 * 1.1 = 110110
        // Regional (central, no change): 110110
        // Rounded to 100: 110100
        assert_eq!(final_price, 110100.0);
    }

    #[test]
    fn test_budget_vehicle_pricing() {
        let market_data = MarketData {
            brand: "Budget".to_string(),
            model: "Car".to_string(),
            average_price: 8000.0,
            market_demand: 0.8,
            seasonal_adjustment: 0.95,
        };

        let final_price = calculate_final_price(8000.0, &market_data, 5.0, "south");

        // Base: 8000
        // Market: 8000 * 0.8 * 0.95 = 6080
        // Margin (5%): 6080 * 1.05 = 6384
        // Regional (south -5%): 6384 * 0.95 = 6064.8
        // Rounded to 100: 6100
        assert_eq!(final_price, 6100.0);
    }

    // ── Price Range Tests ──────────────────────────────────────────────

    #[test]
    fn test_minimum_price_remains_positive() {
        let market_data = MarketData {
            brand: "Old".to_string(),
            model: "Car".to_string(),
            average_price: 500.0,
            market_demand: 0.1,
            seasonal_adjustment: 0.5,
        };

        let final_price = calculate_final_price(1000.0, &market_data, -10.0, "south");

        assert!(final_price >= 0.0);
    }

    #[test]
    fn test_high_end_luxury_pricing() {
        let market_data = MarketData {
            brand: "Rolls".to_string(),
            model: "Royce".to_string(),
            average_price: 250000.0,
            market_demand: 2.0, // Extremely high demand
            seasonal_adjustment: 1.1,
        };

        let final_price = calculate_final_price(200000.0, &market_data, 15.0, "north");

        // Should be very high
        assert!(final_price > 400000.0);
    }

    // ── Price Adjustment Combinations ──────────────────────────────────

    #[test]
    fn test_all_factors_increasing() {
        let market_data = MarketData {
            brand: "Popular".to_string(),
            model: "Model".to_string(),
            average_price: 20000.0,
            market_demand: 1.5, // High
            seasonal_adjustment: 1.1, // High season
        };

        let high_factor = calculate_final_price(20000.0, &market_data, 10.0, "north");

        let market_data_low = MarketData {
            brand: "Popular".to_string(),
            model: "Model".to_string(),
            average_price: 20000.0,
            market_demand: 0.5, // Low
            seasonal_adjustment: 0.9, // Low season
        };

        let low_factor = calculate_final_price(20000.0, &market_data_low, -5.0, "south");

        assert!(high_factor > low_factor);
    }
}
