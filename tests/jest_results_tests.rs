use serde::{Serialize, Deserialize};
use serde_json::json;

// Define a minimal results struct for testing
#[derive(Serialize, Deserialize, Debug)]
pub struct JestResults {
    pub success: bool,
}

#[test]
fn test_jest_results_basic() {
    let json_value = json!({
        "success": true
    });
    
    let jest_results: JestResults = serde_json::from_value(json_value).unwrap();
    assert!(jest_results.success);
}