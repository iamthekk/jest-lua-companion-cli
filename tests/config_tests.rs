use serde::{Serialize, Deserialize};
use serde_json::json;
use std::collections::HashMap;

// Define the Config struct for testing
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub roots: Vec<String>,
}

#[test]
fn test_basic_config_serialization() {
    let config = Config {
        roots: vec!["test".to_string()]
    };
    
    let json = serde_json::to_string(&config).unwrap();
    
    // Simple test to verify serialization works
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["roots"], json!(["test"]));
}