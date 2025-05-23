use jest_lua_companion_cli::config::{Config, ConfigResponse, RunCliOptions};
use serde_json::{json, to_string, from_str};
use std::collections::HashMap;

#[test]
fn test_config_deserialization() {
    let config_json = r#"
    {
        "roots": ["ServerScriptService/TS/Runtime"],
        "runCLI-options": {
            "ci": true,
            "clearMocks": true,
            "testNamePattern": "test.*",
            "testMatch": ["**/*.spec.lua"],
            "verbose": true
        },
        "rojoSourceMapCommand": "rojo sourcemap"
    }
    "#;

    let config: Config = from_str(config_json).expect("Failed to deserialize config");

    assert_eq!(config.roots, vec!["ServerScriptService/TS/Runtime"]);
    assert_eq!(config.rojo_source_map_command, Some("rojo sourcemap".to_string()));
    
    let run_cli_options = config.run_cli_options.expect("RunCliOptions should exist");
    assert_eq!(run_cli_options.ci, Some(true));
    assert_eq!(run_cli_options.clear_mocks, Some(true));
    assert_eq!(run_cli_options.test_name_pattern, Some("test.*".to_string()));
    assert_eq!(run_cli_options.test_match, Some(vec!["**/*.spec.lua".to_string()]));
    assert_eq!(run_cli_options.verbose, Some(true));
}

#[test]
fn test_config_serialization() {
    let mut run_cli_options = RunCliOptions {
        ci: Some(true),
        clear_mocks: Some(true),
        debug: None,
        expand: None,
        json: None,
        list_tests: None,
        no_stack_trace: None,
        old_function_spying: None,
        pass_with_no_tests: None,
        reset_mocks: None,
        setup_files: Some(vec!["setup.lua".to_string()]),
        show_config: None,
        test_match: Some(vec!["**/*.spec.lua".to_string()]),
        test_name_pattern: Some("test.*".to_string()),
        test_path_ignore_patterns: None,
        test_path_pattern: None,
        test_timeout: None,
        update_snapshot: None,
        verbose: Some(true),
    };

    let config = Config {
        roots: vec!["ServerScriptService/TS/Runtime".to_string()],
        test_extra_options: None,
        run_cli_options: Some(run_cli_options),
        rojo_source_map_command: Some("rojo sourcemap".to_string()),
    };

    let json = to_string(&config).expect("Failed to serialize config");
    
    // Verify that serialized JSON contains expected fields
    let parsed: serde_json::Value = from_str(&json).expect("Failed to parse serialized JSON");
    assert!(parsed.get("roots").is_some());
    assert!(parsed.get("runCLI-options").is_some());
    assert!(parsed.get("rojoSourceMapCommand").is_some());
    
    let run_cli_options = parsed.get("runCLI-options").unwrap();
    assert_eq!(run_cli_options.get("ci").unwrap(), &json!(true));
    assert_eq!(run_cli_options.get("clearMocks").unwrap(), &json!(true));
    assert_eq!(run_cli_options.get("setupFiles").unwrap(), &json!(["setup.lua"]));
    assert_eq!(run_cli_options.get("testMatch").unwrap(), &json!(["**/*.spec.lua"]));
    assert_eq!(run_cli_options.get("testNamePattern").unwrap(), &json!("test.*"));
    assert_eq!(run_cli_options.get("verbose").unwrap(), &json!(true));
}

#[test]
fn test_empty_runcli_options() {
    let config_json = r#"
    {
        "roots": ["ServerScriptService"],
        "runCLI-options": {}
    }
    "#;

    let config: Config = from_str(config_json).expect("Failed to deserialize config");
    
    let run_cli_options = config.run_cli_options.expect("RunCliOptions should exist");
    assert_eq!(run_cli_options.ci, None);
    assert_eq!(run_cli_options.clear_mocks, None);
    assert_eq!(run_cli_options.test_match, None);
    assert_eq!(run_cli_options.verbose, None);
}

#[test]
fn test_config_response_serialization() {
    let run_cli_options = RunCliOptions {
        ci: Some(true),
        clear_mocks: Some(true),
        debug: None,
        expand: None,
        json: None,
        list_tests: None,
        no_stack_trace: None,
        old_function_spying: None,
        pass_with_no_tests: None,
        reset_mocks: None,
        setup_files: None,
        show_config: None,
        test_match: None,
        test_name_pattern: None,
        test_path_ignore_patterns: None,
        test_path_pattern: None,
        test_timeout: None,
        update_snapshot: None,
        verbose: None,
    };

    let mut extra_options = HashMap::new();
    extra_options.insert("option1".to_string(), json!("value1"));

    let response = ConfigResponse {
        test_roots: vec!["root1".to_string(), "root2".to_string()],
        test_extra_options: extra_options,
        run_cli_options: Some(run_cli_options),
    };

    let json = to_string(&response).expect("Failed to serialize config response");
    
    // Verify serialized format (camelCase fields)
    let parsed: serde_json::Value = from_str(&json).expect("Failed to parse serialized JSON");
    assert!(parsed.get("testRoots").is_some());
    assert!(parsed.get("testExtraOptions").is_some());
    assert!(parsed.get("runCLI-options").is_some());
    
    assert_eq!(parsed.get("testRoots").unwrap(), &json!(["root1", "root2"]));
}