use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Represents the token usage from an OpenCode interaction
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub reasoning: u64,
    pub cache: CacheUsage,
}

/// Represents cache token usage
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CacheUsage {
    pub write: u64,
    pub read: u64,
}

/// Represents a usage part from OpenCode storage
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UsagePart {
    pub id: String,
    #[serde(rename = "messageID")]
    pub message_id: String,
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub tokens: Option<TokenUsage>,
    pub cost: f64,
}

/// Error types for parsing operations
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Failed to read file: {0}")]
    FileReadError(#[from] std::io::Error),
    
    #[error("Invalid JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Parser for OpenCode usage data
pub struct UsageParser;

impl UsageParser {
    /// Parse JSON string into a UsagePart
    /// Returns None if the part doesn't contain token data
    pub fn parse_json(content: &str) -> Result<Option<UsagePart>, ParserError> {
        let part: UsagePart = serde_json::from_str(content)?;
        
        // Return None if the part doesn't have token data
        if part.tokens.is_none() {
            return Ok(None);
        }
        
        Ok(Some(part))
    }
    
    /// Parse a file into a UsagePart
    /// Returns None if the part doesn't contain token data
    pub fn parse_file(path: &Path) -> Result<Option<UsagePart>, ParserError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_json(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: Deserialize complete JSON with all token data
    #[test]
    fn test_deserialize_complete_usage_part() {
        let json = r#"{
            "id": "prt_99ab34631001IcYXFyeEPSdTZM",
            "messageID": "msg_99ab2e8b7001ifpeClFcxb6yzU",
            "sessionID": "ses_6654d2741ffet36HoSwYBXgCnH",
            "type": "step-finish",
            "tokens": {
                "input": 26535,
                "output": 1322,
                "reasoning": 0,
                "cache": {
                    "write": 0,
                    "read": 0
                }
            },
            "cost": 0
        }"#;

        let part: UsagePart = serde_json::from_str(json).expect("Should deserialize");
        
        assert_eq!(part.id, "prt_99ab34631001IcYXFyeEPSdTZM");
        assert_eq!(part.message_id, "msg_99ab2e8b7001ifpeClFcxb6yzU");
        assert_eq!(part.session_id, "ses_6654d2741ffet36HoSwYBXgCnH");
        assert_eq!(part.event_type, "step-finish");
        assert_eq!(part.cost, 0.0);
        
        let tokens = part.tokens.expect("Should have tokens");
        assert_eq!(tokens.input, 26535);
        assert_eq!(tokens.output, 1322);
        assert_eq!(tokens.reasoning, 0);
        assert_eq!(tokens.cache.write, 0);
        assert_eq!(tokens.cache.read, 0);
    }

    // Test 2: Deserialize with cache read tokens
    #[test]
    fn test_deserialize_with_cache_tokens() {
        let json = r#"{
            "id": "prt_test",
            "messageID": "msg_test",
            "sessionID": "ses_test",
            "type": "step-finish",
            "tokens": {
                "input": 25029,
                "output": 1124,
                "reasoning": 0,
                "cache": {
                    "write": 0,
                    "read": 24781
                }
            },
            "cost": 0.5
        }"#;

        let part: UsagePart = serde_json::from_str(json).expect("Should deserialize");
        
        let tokens = part.tokens.expect("Should have tokens");
        assert_eq!(tokens.cache.read, 24781);
        assert_eq!(part.cost, 0.5);
    }

    // Test 3: Deserialize without tokens field (should be None)
    #[test]
    fn test_deserialize_without_tokens() {
        let json = r#"{
            "id": "prt_test",
            "messageID": "msg_test",
            "sessionID": "ses_test",
            "type": "step-start",
            "cost": 0
        }"#;

        let part: UsagePart = serde_json::from_str(json).expect("Should deserialize");
        assert!(part.tokens.is_none());
    }

    // Test 4: Round-trip serialization
    #[test]
    fn test_serialization_round_trip() {
        let original = UsagePart {
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

        let json = serde_json::to_string(&original).expect("Should serialize");
        let deserialized: UsagePart = serde_json::from_str(&json).expect("Should deserialize");
        
        assert_eq!(original, deserialized);
    }

    // Test 5: Parse valid JSON with complete token data
    #[test]
    fn test_parse_valid_usage_part() {
        let json = r#"{
            "id": "prt_test",
            "messageID": "msg_test",
            "sessionID": "ses_test",
            "type": "step-finish",
            "tokens": {
                "input": 100,
                "output": 50,
                "reasoning": 0,
                "cache": {
                    "write": 0,
                    "read": 0
                }
            },
            "cost": 0.1
        }"#;

        let result = UsageParser::parse_json(json).expect("Should parse successfully");
        let part = result.expect("Should have a UsagePart");
        
        assert_eq!(part.id, "prt_test");
        assert!(part.tokens.is_some());
    }

    // Test 6: Parse part without tokens - should return None
    #[test]
    fn test_parse_part_without_tokens() {
        let json = r#"{
            "id": "prt_test",
            "messageID": "msg_test",
            "sessionID": "ses_test",
            "type": "step-start",
            "cost": 0
        }"#;

        let result = UsageParser::parse_json(json).expect("Should parse successfully");
        assert!(result.is_none(), "Should return None for parts without tokens");
    }

    // Test 7: Parse malformed JSON - should return error
    #[test]
    fn test_parse_malformed_json() {
        let json = r#"{ "id": "test", "invalid json }"#;

        let result = UsageParser::parse_json(json);
        assert!(result.is_err(), "Should return error for malformed JSON");
        assert!(matches!(result.unwrap_err(), ParserError::JsonError(_)));
    }

    // Test 8: Parse empty file
    #[test]
    fn test_parse_empty_file() {
        let json = "";

        let result = UsageParser::parse_json(json);
        assert!(result.is_err(), "Should return error for empty content");
    }

    // Test 9: Parse part with zero tokens
    #[test]
    fn test_parse_part_with_zero_tokens() {
        let json = r#"{
            "id": "prt_test",
            "messageID": "msg_test",
            "sessionID": "ses_test",
            "type": "step-finish",
            "tokens": {
                "input": 0,
                "output": 0,
                "reasoning": 0,
                "cache": {
                    "write": 0,
                    "read": 0
                }
            },
            "cost": 0
        }"#;

        let result = UsageParser::parse_json(json).expect("Should parse successfully");
        let part = result.expect("Should have a UsagePart even with zero tokens");
        
        let tokens = part.tokens.expect("Should have tokens struct");
        assert_eq!(tokens.input, 0);
        assert_eq!(tokens.output, 0);
    }

    // Test 10: Parse file with valid JSON
    #[test]
    fn test_parse_file_valid() {
        use std::io::Write;
        use std::fs::File;
        
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_usage_part.json");
        
        let json = r#"{
            "id": "prt_test_file",
            "messageID": "msg_test",
            "sessionID": "ses_test",
            "type": "step-finish",
            "tokens": {
                "input": 100,
                "output": 50,
                "reasoning": 0,
                "cache": {
                    "write": 0,
                    "read": 0
                }
            },
            "cost": 0.1
        }"#;
        
        // Write test file
        let mut file = File::create(&test_file).expect("Should create test file");
        file.write_all(json.as_bytes()).expect("Should write test data");
        drop(file);
        
        // Parse file
        let result = UsageParser::parse_file(&test_file).expect("Should parse file");
        let part = result.expect("Should have a UsagePart");
        
        assert_eq!(part.id, "prt_test_file");
        assert!(part.tokens.is_some());
        
        // Cleanup
        std::fs::remove_file(test_file).ok();
    }

    // Test 11: Parse non-existent file
    #[test]
    fn test_parse_file_nonexistent() {
        let nonexistent_path = std::path::Path::new("/tmp/nonexistent_file_xyz123.json");
        
        let result = UsageParser::parse_file(nonexistent_path);
        assert!(result.is_err(), "Should return error for nonexistent file");
        assert!(matches!(result.unwrap_err(), ParserError::FileReadError(_)));
    }

    // Test 12: Parse real OpenCode data format
    #[test]
    fn test_parse_real_opencode_format() {
        // This is actual data from OpenCode storage
        let json = r#"{
          "id": "prt_99ab34631001IcYXFyeEPSdTZM",
          "messageID": "msg_99ab2e8b7001ifpeClFcxb6yzU",
          "sessionID": "ses_6654d2741ffet36HoSwYBXgCnH",
          "type": "step-finish",
          "tokens": {
            "input": 26535,
            "output": 1322,
            "reasoning": 0,
            "cache": {
              "write": 0,
              "read": 0
            }
          },
          "cost": 0
        }"#;

        let result = UsageParser::parse_json(json).expect("Should parse real OpenCode data");
        let part = result.expect("Should have a UsagePart");
        
        assert_eq!(part.id, "prt_99ab34631001IcYXFyeEPSdTZM");
        assert_eq!(part.message_id, "msg_99ab2e8b7001ifpeClFcxb6yzU");
        assert_eq!(part.session_id, "ses_6654d2741ffet36HoSwYBXgCnH");
        assert_eq!(part.event_type, "step-finish");
        
        let tokens = part.tokens.expect("Should have tokens");
        assert_eq!(tokens.input, 26535);
        assert_eq!(tokens.output, 1322);
        assert_eq!(tokens.reasoning, 0);
        assert_eq!(tokens.cache.write, 0);
        assert_eq!(tokens.cache.read, 0);
        
        assert_eq!(part.cost, 0.0);
    }
}
