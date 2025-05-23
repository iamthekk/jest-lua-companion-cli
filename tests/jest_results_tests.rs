#[path = "../src/jest_results.rs"]
mod jest_results;

use serde_json::json;
use jest_results::*;
use serde_json::json;

#[test]
fn test_jest_results_deserialization() {
    // Create a minimal Jest results JSON string
    let json_str = r#"{
        "globalConfig": {},
        "results": {
            "numPassedTests": 10,
            "numPendingTestSuites": 0,
            "numFailedTests": 1,
            "numFailedTestSuites": 1,
            "numPassedTestSuites": 2,
            "snapshot": {
                "total": 0
            },
            "testResults": [
                {
                    "leaks": false,
                    "perfStats": {
                        "runtime": 1000,
                        "slow": false,
                        "start": 1000,
                        "end": 2000
                    },
                    "skipped": false,
                    "snapshot": {
                        "total": 0
                    },
                    "testFilePath": "test/file1.lua",
                    "numFailingTests": 1,
                    "numPendingTests": 0,
                    "numPassingTests": 5,
                    "numTodoTests": 0,
                    "testResults": [
                        {
                            "failureMessages": ["Test failed"],
                            "failureDetails": [
                                {
                                    "matcherResult": {
                                        "expected": "value1",
                                        "actual": "value2",
                                        "message": "Expected: value1\nReceived: value2"
                                    },
                                    "__stack": "test/file1.lua:42"
                                }
                            ],
                            "numPassingAsserts": 0,
                            "ancestorTitles": ["Suite"],
                            "invocations": 1,
                            "status": "failed",
                            "fullName": "Suite Test 1",
                            "retryReasons": [],
                            "title": "Test 1"
                        }
                    ]
                }
            ],
            "startTime": 1000,
            "openHandles": [],
            "numTotalTests": 11,
            "success": false,
            "numTotalTestSuites": 3,
            "numRuntimeErrorTestSuites": 0,
            "numPendingTests": 0,
            "numTodoTests": 0,
            "wasInterrupted": false
        }
    }"#;

    // Deserialize the JSON
    let result: Result<JestResults, _> = serde_json::from_str(json_str);
    assert!(result.is_ok(), "Failed to deserialize JestResults");
    
    let jest_results = result.unwrap();
    assert_eq!(jest_results.results.num_total_tests, 11);
    assert_eq!(jest_results.results.num_passed_tests, 10);
    assert_eq!(jest_results.results.num_failed_tests, 1);
    
    // Check the test file results
    let test_file = &jest_results.results.test_results[0];
    assert_eq!(test_file.test_file_path, "test/file1.lua");
    assert_eq!(test_file.num_failing_tests, 1);
    assert_eq!(test_file.num_passing_tests, 5);
    
    // Check the individual test result
    let test_result = &test_file.test_results[0];
    assert_eq!(test_result.status, "failed");
    assert_eq!(test_result.title, "Test 1");
    assert_eq!(test_result.ancestor_titles, vec!["Suite"]);
}

#[test]
fn test_print_results_success() {
    // Create a successful test result
    let json_value = json!({
        "globalConfig": {},
        "results": {
            "numPassedTests": 5,
            "numPendingTestSuites": 0,
            "numFailedTests": 0,
            "numFailedTestSuites": 0,
            "numPassedTestSuites": 1,
            "snapshot": {
                "total": 0
            },
            "testResults": [
                {
                    "leaks": false,
                    "perfStats": {
                        "runtime": 1000,
                        "slow": false,
                        "start": 1000,
                        "end": 2000
                    },
                    "skipped": false,
                    "snapshot": {
                        "total": 0
                    },
                    "testFilePath": "test/success.lua",
                    "numFailingTests": 0,
                    "numPendingTests": 0,
                    "numPassingTests": 5,
                    "numTodoTests": 0,
                    "testResults": [
                        {
                            "failureMessages": [],
                            "failureDetails": [],
                            "numPassingAsserts": 1,
                            "ancestorTitles": [],
                            "invocations": 1,
                            "status": "passed",
                            "fullName": "Test 1",
                            "retryReasons": [],
                            "title": "Test 1"
                        }
                    ]
                }
            ],
            "startTime": 1000,
            "openHandles": [],
            "numTotalTests": 5,
            "success": true,
            "numTotalTestSuites": 1,
            "numRuntimeErrorTestSuites": 0,
            "numPendingTests": 0,
            "numTodoTests": 0,
            "wasInterrupted": false
        }
    });

    let jest_results: JestResults = serde_json::from_value(json_value).unwrap();
    
    // Test with only_failures = false
    let success = print_jest_test_results(&jest_results, false, false, None);
    assert!(success, "Expected success to be true for successful tests");
}

#[test]
fn test_print_results_failure() {
    // Create a failed test result
    let json_value = json!({
        "globalConfig": {},
        "results": {
            "numPassedTests": 0,
            "numPendingTestSuites": 0,
            "numFailedTests": 1,
            "numFailedTestSuites": 1,
            "numPassedTestSuites": 0,
            "snapshot": {
                "total": 0
            },
            "testResults": [
                {
                    "leaks": false,
                    "perfStats": {
                        "runtime": 1000,
                        "slow": false,
                        "start": 1000,
                        "end": 2000
                    },
                    "skipped": false,
                    "snapshot": {
                        "total": 0
                    },
                    "testFilePath": "test/failure.lua",
                    "numFailingTests": 1,
                    "numPendingTests": 0,
                    "numPassingTests": 0,
                    "numTodoTests": 0,
                    "testResults": [
                        {
                            "failureMessages": ["Test failed"],
                            "failureDetails": [],
                            "numPassingAsserts": 0,
                            "ancestorTitles": [],
                            "invocations": 1,
                            "status": "failed",
                            "fullName": "Test 1",
                            "retryReasons": [],
                            "title": "Test 1"
                        }
                    ]
                }
            ],
            "startTime": 1000,
            "openHandles": [],
            "numTotalTests": 1,
            "success": false,
            "numTotalTestSuites": 1,
            "numRuntimeErrorTestSuites": 0,
            "numPendingTests": 0,
            "numTodoTests": 0,
            "wasInterrupted": false
        }
    });

    let jest_results: JestResults = serde_json::from_value(json_value).unwrap();
    
    // Test with only_failures = true
    let success = print_jest_test_results(&jest_results, true, false, None);
    assert!(!success, "Expected success to be false for failed tests");
}

#[test]
fn test_runtime_error_tests() {
    // Create a test result with runtime errors
    let json_value = json!({
        "globalConfig": {},
        "results": {
            "numPassedTests": 0,
            "numPendingTestSuites": 0,
            "numFailedTests": 0,
            "numFailedTestSuites": 1,
            "numPassedTestSuites": 0,
            "snapshot": {
                "total": 0
            },
            "testResults": [
                {
                    "leaks": false,
                    "perfStats": {
                        "runtime": 1000,
                        "slow": false,
                        "start": 1000,
                        "end": 2000
                    },
                    "skipped": false,
                    "snapshot": {
                        "total": 0
                    },
                    "testFilePath": "test/runtime_error.lua",
                    "numFailingTests": 0,
                    "numPendingTests": 0,
                    "numPassingTests": 0,
                    "numTodoTests": 0,
                    "failureMessage": "Runtime Error: Something went wrong",
                    "testResults": []
                }
            ],
            "startTime": 1000,
            "openHandles": [],
            "numTotalTests": 0,
            "success": false,
            "numTotalTestSuites": 1,
            "numRuntimeErrorTestSuites": 1,
            "numPendingTests": 0,
            "numTodoTests": 0,
            "wasInterrupted": false
        }
    });

    let jest_results: JestResults = serde_json::from_value(json_value).unwrap();
    
    // Test with runtime errors
    let success = print_jest_test_results(&jest_results, false, false, None);
    assert!(!success, "Expected success to be false for runtime errors");
}