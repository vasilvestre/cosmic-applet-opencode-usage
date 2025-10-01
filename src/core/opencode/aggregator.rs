use crate::core::opencode::parser::UsagePart;
use std::time::SystemTime;

/// Aggregated usage metrics from OpenCode
#[derive(Debug, Clone, PartialEq)]
pub struct UsageMetrics {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_reasoning_tokens: u64,
    pub total_cache_write_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cost: f64,
    pub interaction_count: usize,
    pub timestamp: SystemTime,
}

/// Aggregates usage parts into metrics
pub struct UsageAggregator {
    total_input_tokens: u64,
    total_output_tokens: u64,
    total_reasoning_tokens: u64,
    total_cache_write_tokens: u64,
    total_cache_read_tokens: u64,
    total_cost: f64,
    interaction_count: usize,
}

impl UsageAggregator {
    /// Create a new aggregator
    pub fn new() -> Self {
        Self {
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_reasoning_tokens: 0,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 0.0,
            interaction_count: 0,
        }
    }

    /// Add a usage part to the aggregation
    pub fn add_part(&mut self, part: &UsagePart) {
        // Only aggregate parts that have token data
        if let Some(tokens) = &part.tokens {
            self.total_input_tokens += tokens.input;
            self.total_output_tokens += tokens.output;
            self.total_reasoning_tokens += tokens.reasoning;
            self.total_cache_write_tokens += tokens.cache.write;
            self.total_cache_read_tokens += tokens.cache.read;
            self.total_cost += part.cost;
            self.interaction_count += 1;
        }
    }

    /// Finalize and return the aggregated metrics
    pub fn finalize(self) -> UsageMetrics {
        UsageMetrics {
            total_input_tokens: self.total_input_tokens,
            total_output_tokens: self.total_output_tokens,
            total_reasoning_tokens: self.total_reasoning_tokens,
            total_cache_write_tokens: self.total_cache_write_tokens,
            total_cache_read_tokens: self.total_cache_read_tokens,
            total_cost: self.total_cost,
            interaction_count: self.interaction_count,
            timestamp: SystemTime::now(),
        }
    }
}

impl Default for UsageAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::opencode::parser::{CacheUsage, TokenUsage};

    // Test 1: Aggregate a single part correctly
    #[test]
    fn test_aggregate_single_part() {
        let mut aggregator = UsageAggregator::new();
        
        let part = UsagePart {
            id: "prt_test".to_string(),
            message_id: "msg_test".to_string(),
            session_id: "ses_test".to_string(),
            event_type: "step-finish".to_string(),
            tokens: Some(TokenUsage {
                input: 100,
                output: 50,
                reasoning: 10,
                cache: CacheUsage {
                    write: 5,
                    read: 15,
                },
            }),
            cost: 0.25,
        };
        
        aggregator.add_part(&part);
        let metrics = aggregator.finalize();
        
        assert_eq!(metrics.total_input_tokens, 100);
        assert_eq!(metrics.total_output_tokens, 50);
        assert_eq!(metrics.total_reasoning_tokens, 10);
        assert_eq!(metrics.total_cache_write_tokens, 5);
        assert_eq!(metrics.total_cache_read_tokens, 15);
        assert_eq!(metrics.total_cost, 0.25);
        assert_eq!(metrics.interaction_count, 1);
    }

    // Test 2: Aggregate multiple parts
    #[test]
    fn test_aggregate_multiple_parts() {
        let mut aggregator = UsageAggregator::new();
        
        let part1 = UsagePart {
            id: "prt_test1".to_string(),
            message_id: "msg_test1".to_string(),
            session_id: "ses_test".to_string(),
            event_type: "step-finish".to_string(),
            tokens: Some(TokenUsage {
                input: 100,
                output: 50,
                reasoning: 10,
                cache: CacheUsage {
                    write: 5,
                    read: 15,
                },
            }),
            cost: 0.25,
        };
        
        let part2 = UsagePart {
            id: "prt_test2".to_string(),
            message_id: "msg_test2".to_string(),
            session_id: "ses_test".to_string(),
            event_type: "step-finish".to_string(),
            tokens: Some(TokenUsage {
                input: 200,
                output: 100,
                reasoning: 20,
                cache: CacheUsage {
                    write: 10,
                    read: 30,
                },
            }),
            cost: 0.50,
        };
        
        let part3 = UsagePart {
            id: "prt_test3".to_string(),
            message_id: "msg_test3".to_string(),
            session_id: "ses_test".to_string(),
            event_type: "step-finish".to_string(),
            tokens: Some(TokenUsage {
                input: 50,
                output: 25,
                reasoning: 5,
                cache: CacheUsage {
                    write: 2,
                    read: 8,
                },
            }),
            cost: 0.10,
        };
        
        aggregator.add_part(&part1);
        aggregator.add_part(&part2);
        aggregator.add_part(&part3);
        
        let metrics = aggregator.finalize();
        
        assert_eq!(metrics.total_input_tokens, 350);
        assert_eq!(metrics.total_output_tokens, 175);
        assert_eq!(metrics.total_reasoning_tokens, 35);
        assert_eq!(metrics.total_cache_write_tokens, 17);
        assert_eq!(metrics.total_cache_read_tokens, 53);
        assert_eq!(metrics.total_cost, 0.85);
        assert_eq!(metrics.interaction_count, 3);
    }

    // Test 3: Handle empty aggregator
    #[test]
    fn test_aggregate_empty() {
        let aggregator = UsageAggregator::new();
        let metrics = aggregator.finalize();
        
        assert_eq!(metrics.total_input_tokens, 0);
        assert_eq!(metrics.total_output_tokens, 0);
        assert_eq!(metrics.total_reasoning_tokens, 0);
        assert_eq!(metrics.total_cache_write_tokens, 0);
        assert_eq!(metrics.total_cache_read_tokens, 0);
        assert_eq!(metrics.total_cost, 0.0);
        assert_eq!(metrics.interaction_count, 0);
    }

    // Test 4: Correctly count cache tokens from real OpenCode data
    #[test]
    fn test_aggregate_with_cache_tokens() {
        let mut aggregator = UsageAggregator::new();
        
        // Simulating real OpenCode data with cache reads
        let part = UsagePart {
            id: "prt_99ab34631001IcYXFyeEPSdTZM".to_string(),
            message_id: "msg_99ab2e8b7001ifpeClFcxb6yzU".to_string(),
            session_id: "ses_6654d2741ffet36HoSwYBXgCnH".to_string(),
            event_type: "step-finish".to_string(),
            tokens: Some(TokenUsage {
                input: 26535,
                output: 1322,
                reasoning: 0,
                cache: CacheUsage {
                    write: 0,
                    read: 24781,
                },
            }),
            cost: 0.0,
        };
        
        aggregator.add_part(&part);
        let metrics = aggregator.finalize();
        
        assert_eq!(metrics.total_input_tokens, 26535);
        assert_eq!(metrics.total_output_tokens, 1322);
        assert_eq!(metrics.total_cache_read_tokens, 24781);
        assert_eq!(metrics.total_cache_write_tokens, 0);
        assert_eq!(metrics.interaction_count, 1);
    }

    // Test 5: Count interactions correctly (one per part with tokens)
    #[test]
    fn test_interaction_counting() {
        let mut aggregator = UsageAggregator::new();
        
        // Add 5 parts with tokens
        for i in 0..5 {
            let part = UsagePart {
                id: format!("prt_test{}", i),
                message_id: format!("msg_test{}", i),
                session_id: "ses_test".to_string(),
                event_type: "step-finish".to_string(),
                tokens: Some(TokenUsage {
                    input: 100,
                    output: 50,
                    reasoning: 0,
                    cache: CacheUsage {
                        write: 0,
                        read: 0,
                    },
                }),
                cost: 0.1,
            };
            aggregator.add_part(&part);
        }
        
        let metrics = aggregator.finalize();
        assert_eq!(metrics.interaction_count, 5);
    }

    // Test 6: Accumulate costs accurately
    #[test]
    fn test_cost_accumulation() {
        let mut aggregator = UsageAggregator::new();
        
        let part1 = UsagePart {
            id: "prt_test1".to_string(),
            message_id: "msg_test1".to_string(),
            session_id: "ses_test".to_string(),
            event_type: "step-finish".to_string(),
            tokens: Some(TokenUsage {
                input: 100,
                output: 50,
                reasoning: 0,
                cache: CacheUsage {
                    write: 0,
                    read: 0,
                },
            }),
            cost: 0.123,
        };
        
        let part2 = UsagePart {
            id: "prt_test2".to_string(),
            message_id: "msg_test2".to_string(),
            session_id: "ses_test".to_string(),
            event_type: "step-finish".to_string(),
            tokens: Some(TokenUsage {
                input: 200,
                output: 100,
                reasoning: 0,
                cache: CacheUsage {
                    write: 0,
                    read: 0,
                },
            }),
            cost: 0.456,
        };
        
        aggregator.add_part(&part1);
        aggregator.add_part(&part2);
        
        let metrics = aggregator.finalize();
        
        // Use approx comparison for floating point
        assert!((metrics.total_cost - 0.579).abs() < 0.0001);
    }

    // Test 7: Handle parts without tokens (should not count as interaction)
    #[test]
    fn test_skip_parts_without_tokens() {
        let mut aggregator = UsageAggregator::new();
        
        let part_without_tokens = UsagePart {
            id: "prt_test".to_string(),
            message_id: "msg_test".to_string(),
            session_id: "ses_test".to_string(),
            event_type: "step-start".to_string(),
            tokens: None,
            cost: 0.0,
        };
        
        aggregator.add_part(&part_without_tokens);
        
        let metrics = aggregator.finalize();
        
        assert_eq!(metrics.total_input_tokens, 0);
        assert_eq!(metrics.interaction_count, 0);
    }

    // Test 8: Timestamp is set when finalized
    #[test]
    fn test_timestamp_set_on_finalize() {
        let aggregator = UsageAggregator::new();
        let before = SystemTime::now();
        let metrics = aggregator.finalize();
        let after = SystemTime::now();
        
        // Timestamp should be between before and after
        assert!(metrics.timestamp >= before);
        assert!(metrics.timestamp <= after);
    }
}
