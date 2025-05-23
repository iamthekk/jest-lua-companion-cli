use crate::rojo_sourcemap::RojoSourceMap;
use console::style;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub failure_messages: Vec<String>,
    pub failure_details: Vec<serde_json::Value>,
    pub num_passing_asserts: u32,
    #[serde(default)]
    pub duration: Option<u32>,
    pub ancestor_titles: Vec<String>,
    pub invocations: u32,
    pub status: String,
    pub full_name: String,
    pub retry_reasons: Vec<String>,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestFile {
    pub leaks: bool,
    pub perf_stats: PerfStats,
    pub skipped: bool,
    pub snapshot: SnapshotStats,
    #[serde(default)]
    pub display_name: Option<DisplayName>,
    #[serde(default)]
    pub open_handles: Vec<serde_json::Value>,
    #[serde(default)]
    pub test_results: Vec<TestResult>,
    pub test_file_path: String,
    pub num_failing_tests: u32,
    pub num_pending_tests: u32,
    pub num_passing_tests: u32,
    pub num_todo_tests: u32,
    #[serde(default)]
    pub failure_message: Option<String>,
    #[serde(default)]
    pub test_exec_error: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PerfStats {
    pub runtime: u64,
    pub slow: bool,
    pub start: u64,
    pub end: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotStats {
    #[serde(default)]
    pub updated: u32,
    #[serde(default)]
    pub unmatched: u32,
    #[serde(default)]
    pub file_deleted: bool,
    #[serde(default)]
    pub unchecked_keys: Vec<String>,
    #[serde(default)]
    pub unchecked: u32,
    #[serde(default)]
    pub added: u32,
    #[serde(default)]
    pub matched: u32,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub files_added: u32,
    #[serde(default)]
    pub files_removed_list: Vec<String>,
    #[serde(default)]
    pub failure: bool,
    #[serde(default)]
    pub files_updated: u32,
    #[serde(default)]
    pub files_unmatched: u32,
    #[serde(default)]
    pub files_removed: u32,
    #[serde(default)]
    pub unchecked_keys_by_file: Vec<String>,
    #[serde(default)]
    pub did_update: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DisplayName {
    pub color: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JestResults {
    pub global_config: serde_json::Value,
    pub results: Results,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Results {
    pub num_passed_tests: u32,
    pub num_pending_test_suites: u32,
    pub num_failed_tests: u32,
    pub num_failed_test_suites: u32,
    pub num_passed_test_suites: u32,
    pub snapshot: SnapshotStats,
    pub test_results: Vec<TestFile>,
    pub start_time: u64,
    pub open_handles: Vec<serde_json::Value>,
    pub num_total_tests: u32,
    pub success: bool,
    pub num_total_test_suites: u32,
    pub num_runtime_error_test_suites: u32,
    pub num_pending_tests: u32,
    pub num_todo_tests: u32,
    pub was_interrupted: bool,
}

pub fn print_jest_test_results(
    results: &JestResults,
    only_failures: bool,
    _debug: bool,
    sourcemap: Option<&RojoSourceMap>,
) -> bool {
    let mut all_success = true;

    // 检查是否有运行时错误测试套件
    if results.results.num_runtime_error_test_suites > 0 {
        eprintln!(
            "{}",
            style(format!(
                "检测到{}个运行时错误测试套件",
                results.results.num_runtime_error_test_suites
            ))
            .red()
        );

        // 输出有运行时错误的测试文件
        for test_file in &results.results.test_results {
            if let Some(failure_message) = &test_file.failure_message {
                eprintln!(
                    "{} {} {}",
                    style("FAIL").red(),
                    style("🚫").red(),
                    style(&test_file.test_file_path).cyan()
                );
                eprintln!("{}", style(failure_message).red());
                eprintln!();
            }
        }

        // 运行时错误被视为失败
        eprintln!(
            "Test Suites: {} failed, {} passed, {} total",
            style(results.results.num_failed_test_suites).red(),
            results.results.num_passed_test_suites,
            results.results.num_total_test_suites
        );

        return false;
    }

    // 按文件收集测试结果
    let mut tests_by_file = std::collections::HashMap::new();

    // 收集所有失败测试摘要
    let mut all_failing_tests = Vec::new();

    for test_file in &results.results.test_results {
        // 如果测试文件被跳过且没有失败的测试，则不显示
        if test_file.skipped && test_file.num_failing_tests == 0 {
            continue;
        }

        let file_path = &test_file.test_file_path;

        let mut has_failure = false;
        let mut failure_details = Vec::new();

        // 检查文件中的所有测试
        for test_result in &test_file.test_results {
            if test_result.status == "failed" {
                has_failure = true;
                all_success = false;

                // 收集失败信息用于所有失败测试摘要
                all_failing_tests.push((file_path, test_result));

                // 收集失败信息
                let test_name = if test_result.ancestor_titles.is_empty() {
                    test_result.title.clone()
                } else {
                    format!(
                        "{} › {}",
                        test_result.ancestor_titles.join(" › "),
                        test_result.title
                    )
                };

                let mut failure_detail = Vec::new();
                failure_detail.push(format!("    ● {}", test_name));
                failure_detail.push("".to_string());

                // 尝试从failureDetails中获取更详细的错误信息
                if !test_result.failure_details.is_empty() {
                    for detail in &test_result.failure_details {
                        // 提取匹配结果信息
                        if let Some(matcher_result) = detail.get("matcherResult") {
                            if let (Some(expected), Some(actual)) =
                                (matcher_result.get("expected"), matcher_result.get("actual"))
                            {
                                // 尝试提取message
                                if let Some(message) = matcher_result.get("message") {
                                    if let Some(msg_str) = message.as_str() {
                                        for line in msg_str.lines() {
                                            // 为Expected和Received添加颜色
                                            if line.contains("Expected:") {
                                                let parts: Vec<&str> =
                                                    line.splitn(2, ":").collect();
                                                if parts.len() > 1 {
                                                    failure_detail.push(format!(
                                                        "    {}: {}",
                                                        parts[0],
                                                        style(parts[1].trim()).green()
                                                    ));
                                                } else {
                                                    failure_detail.push(format!("    {}", line));
                                                }
                                            } else if line.contains("Received:") {
                                                let parts: Vec<&str> =
                                                    line.splitn(2, ":").collect();
                                                if parts.len() > 1 {
                                                    failure_detail.push(format!(
                                                        "    {}: {}",
                                                        parts[0],
                                                        style(parts[1].trim()).red()
                                                    ));
                                                } else {
                                                    failure_detail.push(format!("    {}", line));
                                                }
                                            } else {
                                                failure_detail.push(format!("    {}", line));
                                            }
                                        }

                                        // 在 Expected/Received 信息后添加空行
                                        failure_detail.push("".to_string());
                                    }
                                } else {
                                    // 如果没有message字段，则构建一个
                                    failure_detail.push(format!(
                                        "    expect({}).toBe({}) -- Object.is equality",
                                        style(format!("{}", actual)).red(),
                                        style(format!("{}", expected)).green()
                                    ));
                                    failure_detail.push("".to_string());
                                    failure_detail.push(format!(
                                        "    Expected: {}",
                                        style(format!("{}", expected)).green()
                                    ));
                                    failure_detail.push(format!(
                                        "    Received: {}",
                                        style(format!("{}", actual)).red()
                                    ));

                                    // 添加一个空行，在期望值和实际值后再显示堆栈
                                    failure_detail.push("".to_string());
                                }

                                // 直接从 __stack 获取堆栈信息
                                if let Some(stack) = detail.get("__stack") {
                                    if let Some(stack_str) = stack.as_str() {
                                        // 如果有sourcemap，转换堆栈信息
                                        let final_stack_str = if let Some(sm) = sourcemap {
                                            crate::rojo_sourcemap::convert_stack_trace_text(
                                                stack_str, sm,
                                            )
                                        } else {
                                            stack_str.to_string()
                                        };

                                        // 输出堆栈信息
                                        for stack_line in final_stack_str.lines() {
                                            failure_detail.push(format!("    {}", stack_line));
                                        }
                                        // 在stack结尾添加两个空行
                                        failure_detail.push("".to_string());
                                        failure_detail.push("".to_string());
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // 如果没有详细信息，则使用简单的失败消息
                    for msg in &test_result.failure_messages {
                        failure_detail.push(format!("    {}", msg));
                    }
                }

                failure_details.push(failure_detail);
            }
        }

        // 只保留我们需要的测试文件
        if !only_failures || has_failure {
            tests_by_file.insert(file_path, (has_failure, failure_details));
        }
    }

    // 按文件路径排序，失败的文件排在最后
    let mut file_paths: Vec<&&String> = tests_by_file.keys().collect();
    file_paths.sort_by(|a, b| {
        let a_has_failure = tests_by_file.get(*a).unwrap().0;
        let b_has_failure = tests_by_file.get(*b).unwrap().0;

        if a_has_failure && !b_has_failure {
            std::cmp::Ordering::Greater
        } else if !a_has_failure && b_has_failure {
            std::cmp::Ordering::Less
        } else {
            a.cmp(b)
        }
    });

    // 打印各个文件的测试结果
    let mut prev_has_failure = false;
    for file_path in file_paths {
        let (has_failure, failure_details) = &tests_by_file[file_path];

        // 如果当前文件是失败的，而前一个文件是成功的，添加两行空行
        if *has_failure && !prev_has_failure {
            println!();
            println!();
        }
        prev_has_failure = *has_failure;

        // 打印文件状态
        if *has_failure {
            println!("   {}   🟢 SERVER  {}", style("FAIL").red(), file_path);

            // 打印失败详情
            for (index, detail) in failure_details.iter().enumerate() {
                // 如果不是第一个测试错误，重新打印文件路径
                if index > 0 {
                    println!();
                    println!("   {}   🟢 SERVER  {}", style("FAIL").red(), file_path);
                }

                for line in detail {
                    println!("{}", line);
                }
            }
        } else {
            println!("   {}   🟢 SERVER  {}", style("PASS").green(), file_path);
        }
    }

    // 输出测试结果摘要
    eprintln!(
        "Test Suites: {} failed, {} passed, {} total",
        style(results.results.num_failed_test_suites).red(),
        results.results.num_passed_test_suites,
        results.results.num_total_test_suites
    );
    eprintln!(
        "Tests:       {} failed, {} passed, {} total",
        style(results.results.num_failed_tests).red(),
        results.results.num_passed_tests,
        results.results.num_total_tests
    );
    eprintln!("Snapshots:   {} total", results.results.snapshot.total);

    // 时间计算 - 使用测试文件的运行时间
    let time_display = if let Some(first_file) = results
        .results
        .test_results
        .iter()
        .find(|tf| tf.perf_stats.end > 0 && tf.perf_stats.start > 0)
    {
        let runtime = (first_file.perf_stats.end - first_file.perf_stats.start) as f64;
        runtime / 1000.0
    } else {
        // 回退到使用第一个测试文件的运行时间
        results
            .results
            .test_results
            .first()
            .map(|tf| tf.perf_stats.runtime as f64 / 1000.0)
            .unwrap_or(0.0)
    };

    eprintln!("Time:        {:.3} s", time_display);

    all_success
}

#[cfg(test)]
mod tests {
    use super::*;
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
                        "testResults": []
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
}
