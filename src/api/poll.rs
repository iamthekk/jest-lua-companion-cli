use crate::{
    config::ConfigResponse,
    state::{AppState, Place},
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    Json,
};
use console::style;
use std::sync::Arc;

pub async fn poll(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Json<ConfigResponse>, StatusCode> {
    let headers_map = req.headers();
    let place_guid = headers_map
        .get("place-guid")
        .and_then(|hv| hv.to_str().ok())
        .unwrap();
    let place_name = headers_map
        .get("place-name")
        .and_then(|hv| hv.to_str().ok())
        .unwrap();
    let place_id_str = headers_map
        .get("place-id")
        .and_then(|hv| hv.to_str().ok())
        .unwrap();
    let place_id = place_id_str.parse::<u64>().unwrap();

    let place = Place {
        name: place_name.to_string(),
        id: place_id,
    };

    // 存储place信息，无论它是否是active_place
    state.places.insert(place_guid.to_string(), place.clone());

    let mut active_place_lock = state.active_place.lock().await;

    // 如果没有active_place，则将当前place设置为active_place
    if active_place_lock.is_none() {
        if state.debug {
            eprintln!(
                "{}",
                style(format!(
                    "设置place {} ({}) 为活跃place",
                    place.name, place.id
                ))
                .green()
            );
        }
        active_place_lock.replace(place_guid.to_string());
    }

    // 检查当前place是否是active_place
    match active_place_lock.as_deref() {
        Some(active_place) if active_place == place_guid => {
            if state.debug {
                eprintln!(
                    "{}",
                    style(format!("向活跃place {} 发送配置", place_guid)).dim()
                );

                // 打印详细的配置内容
                eprintln!("发送的配置详情:");
                eprintln!("  test_roots: {:?}", state.config.roots);

                if let Some(run_cli_options) = &state.config.run_cli_options {
                    eprintln!("  runCLI-options:");
                    if let Some(ci) = run_cli_options.ci {
                        eprintln!("    ci: {}", ci);
                    }
                    if let Some(clear_mocks) = run_cli_options.clear_mocks {
                        eprintln!("    clearMocks: {}", clear_mocks);
                    }
                    if let Some(debug) = run_cli_options.debug {
                        eprintln!("    debug: {}", debug);
                    }
                    if let Some(expand) = run_cli_options.expand {
                        eprintln!("    expand: {}", expand);
                    }
                    if let Some(json) = run_cli_options.json {
                        eprintln!("    json: {}", json);
                    }
                    if let Some(list_tests) = run_cli_options.list_tests {
                        eprintln!("    listTests: {}", list_tests);
                    }
                    // ... 其他选项 ...
                    if let Some(test_match) = &run_cli_options.test_match {
                        eprintln!("    testMatch: {:?}", test_match);
                    }
                    if let Some(test_name_pattern) = &run_cli_options.test_name_pattern {
                        eprintln!("    testNamePattern: {}", test_name_pattern);
                    }
                } else {
                    eprintln!("  runCLI-options: None");
                }
            }
            Ok(Json(ConfigResponse {
                test_roots: state.config.roots.clone(),
                test_extra_options: state.config.test_extra_options.clone().unwrap_or_default(),
                run_cli_options: state.config.run_cli_options.clone(),
            }))
        }
        _ => {
            // 即使不是活跃place，也返回OK，但使用空的test_roots以避免运行测试
            if state.debug {
                eprintln!(
                    "{}",
                    style(format!(
                        "收到来自非活跃place {} 的请求，返回空配置",
                        place_guid
                    ))
                    .yellow()
                );
            }
            Ok(Json(ConfigResponse {
                test_roots: Vec::new(),
                test_extra_options: state.config.test_extra_options.clone().unwrap_or_default(),
                run_cli_options: state.config.run_cli_options.clone(),
            }))
        }
    }
}
