use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use config::{Config, RunCliOptions};
use console::style;
use std::{net::SocketAddr, sync::Arc};
use tokio::fs::read_to_string;

mod api;
mod config;
mod jest_results;
mod rojo_sourcemap;
mod state;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    pub only_print_failures: bool,

    #[arg(long)]
    pub jest_results_file: Option<String>,

    #[arg(long, default_value = "false")]
    pub use_rsp_json: bool,

    #[arg(short, long, default_value = "false")]
    pub debug: bool,

    // Jest测试选项
    #[arg(long)]
    pub ci: Option<bool>,

    #[arg(long = "clearMocks")]
    pub clear_mocks: Option<bool>,

    #[arg(long = "jestDebug")]
    pub jest_debug: Option<bool>,

    #[arg(long)]
    pub expand: Option<bool>,

    #[arg(long)]
    pub json: Option<bool>,

    #[arg(long = "listTests")]
    pub list_tests: Option<bool>,

    #[arg(long = "noStackTrace")]
    pub no_stack_trace: Option<bool>,

    #[arg(long = "oldFunctionSpying")]
    pub old_function_spying: Option<bool>,

    #[arg(long = "passWithNoTests")]
    pub pass_with_no_tests: Option<bool>,

    #[arg(long = "resetMocks")]
    pub reset_mocks: Option<bool>,

    #[arg(long = "setupFiles", value_delimiter = '|', num_args = 1.., default_value = "")]
    pub setup_files: Vec<String>,

    #[arg(long = "showConfig")]
    pub show_config: Option<bool>,

    #[arg(long = "testMatch", value_delimiter = '|')]
    pub test_match: Option<Vec<String>>,

    #[arg(long = "testNamePattern", short = 't')]
    pub test_name_pattern: Option<String>,

    #[arg(long = "testPathIgnorePatterns", value_delimiter = '|')]
    pub test_path_ignore_patterns: Option<Vec<String>>,

    #[arg(long = "testPathPattern")]
    pub test_path_pattern: Option<String>,

    #[arg(long = "testTimeout")]
    pub test_timeout: Option<u32>,

    #[arg(long = "updateSnapshot", short = 'u')]
    pub update_snapshot: Option<bool>,

    #[arg(long)]
    pub verbose: Option<bool>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // 默认启动服务器模式
    let config: Arc<Config> = {
        // 尝试从JSON配置文件加载
        let mut config = match read_to_string("jest-lua-companion.config.json").await {
            Ok(contents) => {
                if cli.debug {
                    eprintln!("成功读取JSON配置文件，内容长度: {}", contents.len());
                    eprintln!("配置文件内容: {}", contents);
                }
                match serde_json::from_str::<Config>(&contents) {
                    Ok(config) => {
                        if cli.debug {
                            eprintln!("成功解析配置文件");
                            if let Some(run_cli_options) = &config.run_cli_options {
                                eprintln!("runCLI-options配置详情:");
                                if let Some(ci) = run_cli_options.ci {
                                    eprintln!("  ci: {}", ci);
                                }
                                if let Some(clear_mocks) = run_cli_options.clear_mocks {
                                    eprintln!("  clearMocks: {}", clear_mocks);
                                }
                                if let Some(debug) = run_cli_options.debug {
                                    eprintln!("  debug: {}", debug);
                                }
                                // ... 其他选项打印
                            } else {
                                eprintln!("未配置runCLI-options");
                            }
                        }
                        config
                    }
                    Err(e) => {
                        eprintln!(
                            "{}",
                            style(format!("解析 jest-lua-companion.config.json 失败: {}", e))
                                .red()
                                .bold()
                        );
                        eprintln!("{}", style("配置文件格式错误，程序退出").red().bold());
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    style(format!("无法读取JSON配置文件: {}", e)).red().bold()
                );
                eprintln!("{}", style("配置文件不存在，程序退出").red().bold());
                std::process::exit(1);
            }
        };

        // 从命令行参数创建RunCliOptions
        let cmd_run_cli_options = create_run_cli_options_from_args(&cli);

        // 如果有命令行参数，合并到配置中
        if !cmd_run_cli_options.is_empty() {
            // 合并或创建run_cli_options
            let merged_options = match &mut config.run_cli_options {
                Some(options) => {
                    // 合并选项
                    merge_run_cli_options(options, &cmd_run_cli_options);
                    options
                }
                None => {
                    // 创建新的选项
                    config.run_cli_options = Some(cmd_run_cli_options);
                    config.run_cli_options.as_mut().unwrap()
                }
            };

            if cli.debug {
                eprintln!("合并命令行参数后的runCLI-options:");
                if let Some(ci) = merged_options.ci {
                    eprintln!("  ci: {}", ci);
                }
                if let Some(clear_mocks) = merged_options.clear_mocks {
                    eprintln!("  clearMocks: {}", clear_mocks);
                }
                // 可以添加更多选项的打印...
            }
        }

        Arc::new(config)
    };

    let state = Arc::new(state::AppState {
        config,
        places: dashmap::DashMap::new(),
        active_place: tokio::sync::Mutex::new(None),
        only_log_failures: cli.only_print_failures,
        debug: cli.debug,
    });

    if cli.jest_results_file.is_some() || cli.use_rsp_json {
        eprintln!(
            "{}",
            style("未找到指定的测试结果文件，启动HTTP服务器等待测试结果...").yellow()
        );
    } else {
        eprintln!("{}", style("启动HTTP服务器等待测试结果...").green());

        // 只在调试模式下输出提示信息
        if cli.debug {
            eprintln!(
                "{}",
                style("提示：可以使用 --jest-results-file 参数从文件加载测试结果").dim()
            );
            eprintln!(
                "{}",
                style("提示：可以使用 --use-rsp-json 参数从rsp.json文件加载测试结果").dim()
            );
        }
    }

    let app = Router::new()
        .route("/poll", get(api::poll))
        .route("/logs", post(api::logs))
        .route("/results", post(api::results))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 28860));
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// 从命令行参数创建RunCliOptions
fn create_run_cli_options_from_args(cli: &Cli) -> RunCliOptions {
    RunCliOptions {
        ci: cli.ci,
        clear_mocks: cli.clear_mocks,
        debug: cli.jest_debug,
        expand: cli.expand,
        json: cli.json,
        list_tests: cli.list_tests,
        no_stack_trace: cli.no_stack_trace,
        old_function_spying: cli.old_function_spying,
        pass_with_no_tests: cli.pass_with_no_tests,
        reset_mocks: cli.reset_mocks,
        setup_files: if cli.setup_files.is_empty() {
            None
        } else {
            Some(cli.setup_files.clone())
        },
        show_config: cli.show_config,
        test_match: cli.test_match.clone(),
        test_name_pattern: cli.test_name_pattern.clone(),
        test_path_ignore_patterns: cli.test_path_ignore_patterns.clone(),
        test_path_pattern: cli.test_path_pattern.clone(),
        test_timeout: cli.test_timeout,
        update_snapshot: cli.update_snapshot,
        verbose: cli.verbose,
    }
}

// 检查RunCliOptions是否为空（所有字段都是None）
impl RunCliOptions {
    fn is_empty(&self) -> bool {
        self.ci.is_none()
            && self.clear_mocks.is_none()
            && self.debug.is_none()
            && self.expand.is_none()
            && self.json.is_none()
            && self.list_tests.is_none()
            && self.no_stack_trace.is_none()
            && self.old_function_spying.is_none()
            && self.pass_with_no_tests.is_none()
            && self.reset_mocks.is_none()
            && self.setup_files.is_none()
            && self.show_config.is_none()
            && self.test_match.is_none()
            && self.test_name_pattern.is_none()
            && self.test_path_ignore_patterns.is_none()
            && self.test_path_pattern.is_none()
            && self.test_timeout.is_none()
            && self.update_snapshot.is_none()
            && self.verbose.is_none()
    }
}

// 合并两个RunCliOptions，优先使用第二个的非None值
fn merge_run_cli_options(target: &mut RunCliOptions, source: &RunCliOptions) {
    if let Some(value) = source.ci {
        target.ci = Some(value);
    }
    if let Some(value) = source.clear_mocks {
        target.clear_mocks = Some(value);
    }
    if let Some(value) = source.debug {
        target.debug = Some(value);
    }
    if let Some(value) = source.expand {
        target.expand = Some(value);
    }
    if let Some(value) = source.json {
        target.json = Some(value);
    }
    if let Some(value) = source.list_tests {
        target.list_tests = Some(value);
    }
    if let Some(value) = source.no_stack_trace {
        target.no_stack_trace = Some(value);
    }
    if let Some(value) = source.old_function_spying {
        target.old_function_spying = Some(value);
    }
    if let Some(value) = source.pass_with_no_tests {
        target.pass_with_no_tests = Some(value);
    }
    if let Some(value) = source.reset_mocks {
        target.reset_mocks = Some(value);
    }
    if let Some(ref value) = source.setup_files {
        target.setup_files = Some(value.clone());
    }
    if let Some(value) = source.show_config {
        target.show_config = Some(value);
    }
    if let Some(ref value) = source.test_match {
        target.test_match = Some(value.clone());
    }
    if let Some(ref value) = source.test_name_pattern {
        target.test_name_pattern = Some(value.clone());
    }
    if let Some(ref value) = source.test_path_ignore_patterns {
        target.test_path_ignore_patterns = Some(value.clone());
    }
    if let Some(ref value) = source.test_path_pattern {
        target.test_path_pattern = Some(value.clone());
    }
    if let Some(value) = source.test_timeout {
        target.test_timeout = Some(value);
    }
    if let Some(value) = source.update_snapshot {
        target.update_snapshot = Some(value);
    }
    if let Some(value) = source.verbose {
        target.verbose = Some(value);
    }
}
