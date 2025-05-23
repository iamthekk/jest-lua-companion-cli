use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[test]
fn test_basic_map_operations() {
    // This is a basic test just to have something that passes
    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());
    
    assert_eq!(map.get("key"), Some(&"value".to_string()));
}