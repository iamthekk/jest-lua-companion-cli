#[path = "../src/rojo_sourcemap.rs"]
mod rojo_sourcemap;

use std::collections::HashMap;
use serde_json::from_str;
use rojo_sourcemap::*;

// Test helper functions
impl RojoSourceMap {
    pub fn get_path_map(&self) -> &HashMap<String, String> {
        &self.path_map
    }
    
    pub fn get_path_map_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.path_map
    }
    
    pub fn get_entries(&self) -> &HashMap<String, RojoSourceMapEntry> {
        &self.entries
    }
    
    pub fn create_test_sourcemap(
        entries: HashMap<String, RojoSourceMapEntry>, 
        path_map: HashMap<String, String>
    ) -> Self {
        RojoSourceMap {
            entries,
            path_map,
        }
    }
}

// Implement Clone for RojoSourceMap for tests
impl Clone for RojoSourceMap {
    fn clone(&self) -> Self {
        RojoSourceMap {
            entries: self.entries.clone(),
            path_map: self.path_map.clone(),
        }
    }
}
use std::collections::HashMap;

#[test]
fn test_sourcemap_creation_flat_format() {
    let json = r#"{
        "path1": {
            "path": "C:/path/to/file1.lua",
            "filePaths": ["C:/path/to/file1.lua"]
        },
        "path2": {
            "path": "C:/path/to/file2.lua",
            "filePaths": ["C:/path/to/file2.lua"]
        }
    }"#;

    let sourcemap = RojoSourceMap::new(json).expect("Failed to create sourcemap");
    
    // Check that entries were parsed correctly
    assert!(sourcemap.get_entries().contains_key("path1"));
    assert!(sourcemap.get_entries().contains_key("path2"));
    
    // Check the paths in entries
    assert_eq!(sourcemap.get_entries().get("path1").unwrap().path, "C:/path/to/file1.lua");
    assert_eq!(sourcemap.get_entries().get("path2").unwrap().path, "C:/path/to/file2.lua");
}

#[test]
fn test_sourcemap_creation_tree_format() {
    let json = r#"{
        "name": "DataModel",
        "className": "DataModel",
        "children": [
            {
                "name": "ServerScriptService",
                "className": "ServerScriptService",
                "children": [
                    {
                        "name": "Script",
                        "className": "Script",
                        "children": [],
                        "filePaths": ["C:/path/to/script.lua"]
                    }
                ]
            }
        ]
    }"#;

    let sourcemap = RojoSourceMap::new(json).expect("Failed to create sourcemap");
    
    // Check path_map was created correctly
    assert!(sourcemap.get_path_map().contains_key("DataModel.ServerScriptService.Script"));
    
    // Check the paths in path_map
    assert_eq!(
        sourcemap.get_path_map().get("DataModel.ServerScriptService.Script").unwrap(), 
        "C:/path/to/script.lua"
    );
}

#[test]
fn test_get_file_path() {
    // Create a test sourcemap with both entries and path_map
    let mut entries = HashMap::new();
    entries.insert(
        "ServerScriptService.Script1".to_string(), 
        RojoSourceMapEntry {
            path: "C:/entries/script1.lua".to_string(),
            filePaths: vec!["C:/entries/script1.lua".to_string()]
        }
    );
    
    let mut path_map = HashMap::new();
    path_map.insert(
        "DataModel.ServerScriptService.Script2".to_string(),
        "C:/path_map/script2.lua".to_string()
    );
    path_map.insert(
        "Game.ReplicatedStorage.Module".to_string(),
        "C:/path_map/module.lua".to_string()
    );
    
    let sourcemap = RojoSourceMap::create_test_sourcemap(entries, path_map);
    
    // Test direct matches
    assert_eq!(
        sourcemap.get_file_path("ServerScriptService.Script1"),
        Some("C:/entries/script1.lua".to_string())
    );
    
    assert_eq!(
        sourcemap.get_file_path("DataModel.ServerScriptService.Script2"),
        Some("C:/path_map/script2.lua".to_string())
    );
    
    // Test with added .lua suffix
    let mut sourcemap_with_lua = sourcemap.clone();
    sourcemap_with_lua.get_path_map_mut().insert(
        "Module.lua".to_string(),
        "C:/path_map/module_with_lua.lua".to_string()
    );
    
    assert_eq!(
        sourcemap_with_lua.get_file_path("Module"),
        Some("C:/path_map/module_with_lua.lua".to_string())
    );
    
    // Test with DataModel prefix
    assert_eq!(
        sourcemap.get_file_path("ServerScriptService.Script2"),
        Some("C:/path_map/script2.lua".to_string())
    );
    
    // Test with root project prefix
    assert_eq!(
        sourcemap.get_file_path("ReplicatedStorage.Module"),
        Some("C:/path_map/module.lua".to_string())
    );
    
    // Test with path that doesn't exist
    assert_eq!(
        sourcemap.get_file_path("NonexistentPath"),
        None
    );
}

#[test]
fn test_convert_stack_trace() {
    // Create a test sourcemap
    let mut path_map = HashMap::new();
    path_map.insert(
        "DataModel.ServerScriptService.Script".to_string(),
        "C:/path/to/script.lua".to_string()
    );
    path_map.insert(
        "DataModel.ReplicatedStorage.Module".to_string(),
        "C:/path/to/module.lua".to_string()
    );
    
    let sourcemap = RojoSourceMap::create_test_sourcemap(HashMap::new(), path_map);
    
    // Test basic conversion
    assert_eq!(
        convert_stack_trace("ServerScriptService.Script:42", &sourcemap),
        "C:/path/to/script.lua:42"
    );
    
    // Test with DataModel prefix
    assert_eq!(
        convert_stack_trace("DataModel.ServerScriptService.Script:42", &sourcemap),
        "C:/path/to/script.lua:42"
    );
    
    // Test with path that doesn't exist in sourcemap
    let original_line = "NonexistentPath:42";
    assert_eq!(
        convert_stack_trace(original_line, &sourcemap),
        original_line
    );
}

#[test]
fn test_convert_stack_trace_text() {
    // Create a test sourcemap
    let mut path_map = HashMap::new();
    path_map.insert(
        "DataModel.ServerScriptService.Script1".to_string(),
        "C:/path/to/script1.lua".to_string()
    );
    path_map.insert(
        "DataModel.ServerScriptService.Script2".to_string(),
        "C:/path/to/script2.lua".to_string()
    );
    
    let sourcemap = RojoSourceMap::create_test_sourcemap(HashMap::new(), path_map);
    
    // Test multi-line conversion
    let stack_trace = "ServerScriptService.Script1:42\nServerScriptService.Script2:24\nNonExistentPath:10";
    let expected = "C:/path/to/script1.lua:42\nC:/path/to/script2.lua:24\nNonExistentPath:10";
    
    assert_eq!(
        convert_stack_trace_text(stack_trace, &sourcemap),
        expected
    );
}
