//! Cotation Service business logic tests
//! Tests for vehicle valuation calculations

#[cfg(test)]
mod tests {
    // Mock vehicle data structure
    #[derive(Debug, Clone)]
    struct Vehicle {
        brand: String,
        model: String,
        year: u32,
        mileage: u32,
        condition: String, // excellent, good, fair, poor
    }

    // Price calculation function
    fn calculate_base_price(brand: &str, model: &str) -> f64 {
        match (brand.to_lowercase().as_str(), model.to_lowercase().as_str()) {
            ("toyota", "corolla") => 15000.0,
            ("ford", "focus") => 12000.0,
            ("bmw", "320") => 25000.0,
            ("audi", "a4") => 28000.0,
            _ => 10000.0, // Default for unknown vehicles
        }
    }

    fn apply_year_depreciation(base_price: f64, year: u32) -> f64 {
        let current_year = 2024;
        let age = current_year - year;

        // 15% depreciation for first year, then 10% per year
        let depreciation_rate = if age == 0 {
            0.0
        } else if age == 1 {
            0.15
        } else {
            0.15 + ((age - 1) as f64 * 0.10)
        };

        let total_depreciation = depreciation_rate.min(0.85); // Max 85% depreciation
        base_price * (1.0 - total_depreciation)
    }

    fn apply_mileage_depreciation(price: f64, mileage: u32) -> f64 {
        // $0.10 per km
        let mileage_deduction = (mileage as f64) * 0.10;
        (price - mileage_deduction).max(0.0) // Can't be negative
    }

    fn apply_condition_adjustment(price: f64, condition: &str) -> f64 {
        match condition.to_lowercase().as_str() {
            "excellent" => price * 1.10, // +10%
            "good" => price * 1.0,       // No change
            "fair" => price * 0.85,      // -15%
            "poor" => price * 0.70,      // -30%
            _ => price,                  // Unknown condition, no change
        }
    }

    fn calculate_cotation(vehicle: &Vehicle) -> f64 {
        let base = calculate_base_price(&vehicle.brand, &vehicle.model);
        let year_adjusted = apply_year_depreciation(base, vehicle.year);
        let mileage_adjusted = apply_mileage_depreciation(year_adjusted, vehicle.mileage);
        let final_price = apply_condition_adjustment(mileage_adjusted, &vehicle.condition);

        // Round to nearest 100
        (final_price / 100.0).round() * 100.0
    }

    // ── Base Price Calculation Tests ───────────────────────────────────

    #[test]
    fn test_base_price_known_vehicle() {
        assert_eq!(calculate_base_price("Toyota", "Corolla"), 15000.0);
        assert_eq!(calculate_base_price("Ford", "Focus"), 12000.0);
        assert_eq!(calculate_base_price("BMW", "320"), 25000.0);
    }

    #[test]
    fn test_base_price_unknown_vehicle() {
        assert_eq!(calculate_base_price("Unknown", "Model"), 10000.0);
    }

    #[test]
    fn test_base_price_case_insensitive() {
        assert_eq!(calculate_base_price("toyota", "corolla"), 15000.0);
        assert_eq!(calculate_base_price("TOYOTA", "COROLLA"), 15000.0);
    }

    // ── Year Depreciation Tests ────────────────────────────────────────

    #[test]
    fn test_current_year_vehicle_no_depreciation() {
        let price = 20000.0;
        let current_year = 2024;
        let result = apply_year_depreciation(price, current_year);

        assert_eq!(result, 20000.0);
    }

    #[test]
    fn test_one_year_old_vehicle_15_percent_depreciation() {
        let price = 20000.0;
        let one_year_old = 2023;
        let result = apply_year_depreciation(price, one_year_old);

        assert_eq!(result, 17000.0); // 20000 * 0.85
    }

    #[test]
    fn test_five_year_old_vehicle_depreciation() {
        let price = 20000.0;
        let five_years_old = 2019;
        let result = apply_year_depreciation(price, five_years_old);

        // 15% + 4 * 10% = 55% depreciation
        assert!((result - 9000.0).abs() < 0.01);
    }

    #[test]
    fn test_very_old_vehicle_capped_depreciation() {
        let price = 20000.0;
        let very_old = 1950; // 74 years old
        let result = apply_year_depreciation(price, very_old);

        // Should be capped at 85% depreciation = 15% of original
        assert_eq!(result, 3000.0);
    }

    // ── Mileage Depreciation Tests ─────────────────────────────────────

    #[test]
    fn test_no_mileage_no_deduction() {
        let price = 20000.0;
        let result = apply_mileage_depreciation(price, 0);

        assert_eq!(result, 20000.0);
    }

    #[test]
    fn test_mileage_depreciation_10_cents_per_km() {
        let price = 20000.0;
        let mileage = 100000; // 100k km
        let result = apply_mileage_depreciation(price, mileage);

        assert_eq!(result, 10000.0); // 20000 - 10000
    }

    #[test]
    fn test_mileage_cannot_make_price_negative() {
        let price = 1000.0;
        let mileage = 200000; // Would deduct 20000
        let result = apply_mileage_depreciation(price, mileage);

        assert_eq!(result, 0.0);
    }

    // ── Condition Adjustment Tests ─────────────────────────────────────

    #[test]
    fn test_excellent_condition_plus_10_percent() {
        let price = 10000.0;
        let result = apply_condition_adjustment(price, "excellent");

        assert_eq!(result, 11000.0);
    }

    #[test]
    fn test_good_condition_no_adjustment() {
        let price = 10000.0;
        let result = apply_condition_adjustment(price, "good");

        assert_eq!(result, 10000.0);
    }

    #[test]
    fn test_fair_condition_minus_15_percent() {
        let price = 10000.0;
        let result = apply_condition_adjustment(price, "fair");

        assert_eq!(result, 8500.0);
    }

    #[test]
    fn test_poor_condition_minus_30_percent() {
        let price = 10000.0;
        let result = apply_condition_adjustment(price, "poor");

        assert_eq!(result, 7000.0);
    }

    #[test]
    fn test_condition_case_insensitive() {
        assert_eq!(apply_condition_adjustment(10000.0, "EXCELLENT"), 11000.0);
        assert_eq!(apply_condition_adjustment(10000.0, "Good"), 10000.0);
    }

    // ── Full Cotation Calculation Tests ────────────────────────────────

    #[test]
    fn test_full_cotation_calculation() {
        let vehicle = Vehicle {
            brand: "Toyota".to_string(),
            model: "Corolla".to_string(),
            year: 2023,
            mileage: 50000,
            condition: "good".to_string(),
        };

        let price = calculate_cotation(&vehicle);

        // Base: 15000
        // Year (1 year old): 15000 * 0.85 = 12750
        // Mileage (50k km): 12750 - 5000 = 7750
        // Condition (good): 7750 * 1.0 = 7750
        // Rounded to 100: 7800
        assert_eq!(price, 7800.0);
    }

    #[test]
    fn test_excellent_condition_vehicle() {
        let vehicle = Vehicle {
            brand: "BMW".to_string(),
            model: "320".to_string(),
            year: 2024,
            mileage: 10000,
            condition: "excellent".to_string(),
        };

        let price = calculate_cotation(&vehicle);

        // Base: 25000
        // Year (current): 25000 * 1.0 = 25000
        // Mileage (10k km): 25000 - 1000 = 24000
        // Condition (excellent): 24000 * 1.1 = 26400
        // Rounded: 26400
        assert_eq!(price, 26400.0);
    }

    #[test]
    fn test_poor_condition_old_vehicle_with_high_mileage() {
        let vehicle = Vehicle {
            brand: "Ford".to_string(),
            model: "Focus".to_string(),
            year: 2015,
            mileage: 250000,
            condition: "poor".to_string(),
        };

        let price = calculate_cotation(&vehicle);

        // Base: 12000
        // Year (9 years old): significant depreciation
        // Mileage (250k km): 25000 deduction
        // Condition (poor): 30% deduction
        // Result should be positive but low
        assert!(price > 0.0);
        assert!(price < 5000.0);
    }

    // ── Edge Cases ─────────────────────────────────────────────────────

    #[test]
    fn test_very_low_price_vehicle() {
        let vehicle = Vehicle {
            brand: "Unknown".to_string(),
            model: "Model".to_string(),
            year: 1990,
            mileage: 500000,
            condition: "poor".to_string(),
        };

        let price = calculate_cotation(&vehicle);
        assert!(price >= 0.0);
    }

    #[test]
    fn test_luxury_vehicle_good_condition() {
        let vehicle = Vehicle {
            brand: "Audi".to_string(),
            model: "A4".to_string(),
            year: 2023,
            mileage: 5000,
            condition: "excellent".to_string(),
        };

        let price = calculate_cotation(&vehicle);

        // Should be substantial
        assert!(price > 25000.0);
    }
}
