use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub roots: Vec<String>,
    pub test_extra_options: Option<HashMap<String, Value>>,
    #[serde(rename = "runCLI-options")]
    pub run_cli_options: Option<RunCliOptions>,
    #[serde(
        rename = "rojoSourceMapCommand",
        skip_serializing_if = "Option::is_none"
    )]
    pub rojo_source_map_command: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunCliOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "clearMocks")]
    pub clear_mocks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "listTests")]
    pub list_tests: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "noStackTrace")]
    pub no_stack_trace: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "oldFunctionSpying")]
    pub old_function_spying: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "passWithNoTests")]
    pub pass_with_no_tests: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "resetMocks")]
    pub reset_mocks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "setupFiles")]
    pub setup_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "showConfig")]
    pub show_config: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "testMatch")]
    pub test_match: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "testNamePattern")]
    pub test_name_pattern: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "testPathIgnorePatterns"
    )]
    pub test_path_ignore_patterns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "testPathPattern")]
    pub test_path_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "testTimeout")]
    pub test_timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "updateSnapshot")]
    pub update_snapshot: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbose: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub test_roots: Vec<String>,
    pub test_extra_options: HashMap<String, Value>,
    #[serde(rename = "runCLI-options")]
    pub run_cli_options: Option<RunCliOptions>,
}
