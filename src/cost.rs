use crate::models::{Pricing, Usage};

pub fn estimate_cost(input_tokens: usize, output_tokens: usize, pricing: &Pricing) -> f64 {
    let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_per_million;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output_per_million;

    input_cost + output_cost
}

pub fn calculate_cost(
    usage: Option<&Usage>,
    input_tokens: usize,
    output_tokens: usize,
    config: &Pricing,
) -> (f64, bool) {
    match usage {
        Some(usage) => {
            let cost = estimate_cost(usage.prompt_tokens, usage.completion_tokens, config);

            (cost, true)
        }
        None => {
            let cost = estimate_cost(input_tokens, output_tokens, config);

            (cost, false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pricing_fixture() -> Pricing {
        Pricing {
            model: "test-model".to_string(),
            input_per_million: 10.0,
            output_per_million: 20.0,
        }
    }

    #[test]
    fn estimate_cost_calculates_input_and_output_cost() {
        let pricing = pricing_fixture();

        let cost = estimate_cost(1_000_000, 5_00_000, &pricing);

        assert!((cost - 20.00).abs() < 1e-12);
    }

    #[test]
    fn estimate_cost_returns_zero_for_zero_tokens() {
        let pricing = pricing_fixture();

        let cost = estimate_cost(0, 0, &pricing);

        assert!((cost - 0.0).abs() < 1e-12);
    }

    #[test]
    fn calculate_cost_prefers_provider_usage_when_present() {
        let pricing = pricing_fixture();

        let usage = Usage {
            prompt_tokens: 1_000_000,
            completion_tokens: 500_000,
            total_tokens: 1_500_000,
        };

        let (cost, used_actual_usage) = calculate_cost(Some(&usage), 0, 0, &pricing);

        assert!((cost - 20.0).abs() < 1e-12);
        assert!(used_actual_usage);
    }

    #[test]
    fn calculate_cost_falls_back_to_estimates_when_usage_is_missing() {
        let pricing = pricing_fixture();

        let (cost, used_actual_usage) = calculate_cost(None, 1_000_000, 500_000, &pricing);

        assert!((cost - 20.0).abs() < 1e-12);
        assert!(!used_actual_usage);
    }
}
