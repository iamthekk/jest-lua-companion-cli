#[path = "../src/rojo_sourcemap.rs"]
mod rojo_sourcemap;

use rojo_sourcemap::{RojoSourceMap, convert_stack_trace};

fn main() {
    // This is a simple example of using the convert_stack_trace function
    let sample_json = r#"{"DataModel": {"className": "DataModel", "name": "DataModel", "children": []}}"#;
    let sourcemap = RojoSourceMap::new(sample_json).expect("Failed to create sourcemap");
    
    let stack_trace = "ServerScriptService.TS.Runtime.__tests__.os-lib.jack:58";
    let converted = convert_stack_trace(stack_trace, &sourcemap);
    
    println!("Original: {}", stack_trace);
    println!("Converted: {}", converted);
}