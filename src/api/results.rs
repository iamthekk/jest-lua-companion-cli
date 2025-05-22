use crate::{
    jest_results::{print_jest_test_results, JestResults},
    rojo_sourcemap::{get_sourcemap, RojoSourceMap},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, Json};
use console::style;
use serde_json::Value;
use std::{fs::File, io::Write, process::exit, sync::Arc, time::Duration};
use tokio::{spawn, time::sleep};

pub async fn results(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> StatusCode {
    // 保存接收到的JSON以便调试
    let json_str = serde_json::to_string_pretty(&body).unwrap_or_default();

    if state.debug {
        eprintln!("{}", style("收到测试结果数据").green());

        // 保存到文件以便后续分析 (仅在 debug 模式下)
        if let Ok(mut file) = File::create("received_results.json") {
            let _ = file.write_all(json_str.as_bytes());
            eprintln!(
                "{}",
                style("已保存测试结果到received_results.json文件").dim()
            );
        }
    }

    // 首先尝试解析为Jest结果格式
    let jest_result = serde_json::from_value::<JestResults>(body.clone());

    match jest_result {
        Ok(jest_results) => {
            if state.debug {
                eprintln!("{}", style("成功解析为Jest测试结果格式").green());
            }

            // 获取 Sourcemap
            let sourcemap: Option<RojoSourceMap> =
                if let Some(command) = &state.config.rojo_source_map_command {
                    if state.debug {
                        eprintln!("Sourcemap command: {}", command);
                    }
                    // 尝试从配置文件所在的目录执行命令
                    let config_dir = std::env::current_dir().ok().and_then(|curr_dir| {
                        curr_dir
                            .join("jest-lua-companion.config.json")
                            .parent()
                            .map(|p| p.to_path_buf())
                    });

                    match get_sourcemap(command, config_dir.as_deref().and_then(|p| p.to_str())) {
                        Ok(sm) => {
                            if state.debug {}
                            Some(sm)
                        }
                        Err(e) => {
                            eprintln!("{}", style(format!("获取Rojo Sourcemap失败: {}", e)).red());
                            None
                        }
                    }
                } else {
                    if state.debug {
                        eprintln!("{}", style("未配置Rojo Sourcemap命令").yellow());
                    }
                    None
                };

            let success = print_jest_test_results(
                &jest_results,
                state.only_log_failures,
                state.debug,
                sourcemap.as_ref(),
            );

            // 延迟退出，确保响应先发送
            spawn(async move {
                sleep(Duration::from_millis(100)).await;
                exit(if success { 0 } else { 1 });
            });
        }
        Err(e) => {
            if state.debug {
                eprintln!("{}", style(format!("解析Jest结果失败: {}", e)).red());
                eprintln!("{}", style("尝试查看数据结构...").yellow());

                // 输出接收到的数据的顶级字段，以便了解其结构
                if let Some(obj) = body.as_object() {
                    eprintln!("{}", style("接收到的JSON对象包含以下顶级字段:").yellow());
                    for (key, value) in obj {
                        let type_info = match value {
                            Value::Null => "null".to_string(),
                            Value::Bool(_) => "布尔值".to_string(),
                            Value::Number(_) => "数字".to_string(),
                            Value::String(_) => "字符串".to_string(),
                            Value::Array(arr) => format!("数组({}项)", arr.len()),
                            Value::Object(obj) => format!("对象({}个字段)", obj.len()),
                        };
                        eprintln!("  - {}: {}", key, type_info);
                    }
                }

                // 尝试直接处理原始数据，提取测试结果信息
                if let Some(results) = body.get("results") {
                    if let Some(test_results) = results.get("testResults") {
                        if let Some(arr) = test_results.as_array() {
                            eprintln!(
                                "{}",
                                style(format!("找到测试结果数组，包含{}个测试文件", arr.len()))
                                    .green()
                            );

                            // 输出简单的测试摘要
                            if let Some(passed) = results.get("numPassedTests") {
                                if let Some(failed) = results.get("numFailedTests") {
                                    eprintln!(
                                        "测试摘要: 通过 {}, 失败 {}",
                                        passed.as_u64().unwrap_or(0),
                                        failed.as_u64().unwrap_or(0)
                                    );
                                }
                            }

                            // 检查是否有运行时错误测试套件
                            if let Some(runtime_errors) = results.get("numRuntimeErrorTestSuites") {
                                let error_count = runtime_errors.as_u64().unwrap_or(0);
                                if error_count > 0 {
                                    eprintln!(
                                        "{}",
                                        style(format!("检测到{}个运行时错误测试套件", error_count))
                                            .red()
                                    );

                                    // 输出运行时错误信息
                                    for test_file in arr {
                                        if let Some(failure_message) =
                                            test_file.get("failureMessage")
                                        {
                                            if let Some(message) = failure_message.as_str() {
                                                eprintln!(
                                                    "{}",
                                                    style("测试套件运行时错误:").red().bold()
                                                );
                                                eprintln!("{}", style(message).red());
                                            }
                                        }
                                    }

                                    // 运行时错误也应该被视为失败
                                    spawn(async move {
                                        sleep(Duration::from_millis(100)).await;
                                        exit(1);
                                    });
                                    return StatusCode::OK;
                                }
                            }
                        }
                    }
                }

                eprintln!("{}", style("尽管解析失败，但仍返回成功状态码").yellow());
            } else {
                eprintln!("{}", style("解析测试结果失败").red());
            }
        }
    }

    StatusCode::OK
}
