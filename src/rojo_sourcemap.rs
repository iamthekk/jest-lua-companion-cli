use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct RojoSourceMapEntry {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub filePaths: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct RojoSourceMapNode {
    pub name: String,
    pub className: String,
    #[serde(default)]
    pub filePaths: Vec<String>,
    #[serde(default)]
    pub children: Vec<RojoSourceMapNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RojoSourceMap {
    #[serde(flatten)]
    entries: HashMap<String, RojoSourceMapEntry>,
    // 解析后的完整路径映射
    #[serde(skip)]
    path_map: HashMap<String, String>,
}

impl RojoSourceMap {
    pub fn new(json: &str) -> Result<Self, Box<dyn Error>> {
        // 先尝试解析为展平的格式
        if let Ok(map) = serde_json::from_str::<HashMap<String, RojoSourceMapEntry>>(json) {
            let source_map = RojoSourceMap {
                entries: map,
                path_map: HashMap::new(),
            };
            return Ok(source_map);
        }

        // 如果失败，则尝试解析为树状结构并遍历
        let root_node: RojoSourceMapNode = serde_json::from_str(json)?;
        let mut path_map = HashMap::new();

        // 递归遍历树状结构，构建路径映射
        process_node(&root_node, "", &mut path_map);

        Ok(RojoSourceMap {
            entries: HashMap::new(), // 树状结构不使用entries
            path_map,
        })
    }

    /// 获取DataModel路径对应的本地文件路径
    pub fn get_file_path(&self, datamodel_path: &str) -> Option<String> {
        let datamodel_path = datamodel_path.trim();

        // 1. 首先直接查找完整路径
        if let Some(path) = self.path_map.get(datamodel_path) {
            return Some(path.clone());
        }

        // 2. 尝试添加.lua后缀查找
        let path_with_lua = format!("{}.lua", datamodel_path);
        if let Some(path) = self.path_map.get(&path_with_lua) {
            return Some(path.clone());
        }

        // 3. 如果是旧格式，尝试从entries获取
        if let Some(entry) = self.entries.get(datamodel_path) {
            if !entry.path.is_empty() {
                return Some(entry.path.clone());
            }
            if !entry.filePaths.is_empty() {
                return Some(entry.filePaths[0].clone());
            }
        }

        // 4. 准备添加DataModel前缀
        let datamodel_key_prefix = format!("DataModel.{}", datamodel_path);
        if let Some(path) = self.path_map.get(&datamodel_key_prefix) {
            return Some(path.clone());
        }

        // 5. 尝试添加根项目名称前缀
        if let Some((root_name, _)) = self.path_map.iter().next() {
            if root_name.contains('.') {
                // 取根项目名称
                let root_project = root_name.split('.').next().unwrap_or("");
                if !root_project.is_empty() {
                    let full_path = format!("{}.{}", root_project, datamodel_path);
                    let full_path_lua = format!("{}.{}.lua", root_project, datamodel_path);

                    if let Some(path) = self.path_map.get(&full_path) {
                        return Some(path.clone());
                    }

                    if let Some(path) = self.path_map.get(&full_path_lua) {
                        return Some(path.clone());
                    }
                }
            }
        }

        // 6. 处理部分匹配
        let path_parts: Vec<&str> = datamodel_path.split('.').collect();
        if path_parts.len() >= 2 {
            let target_part = path_parts[path_parts.len() - 1]; // 最后一部分
            let parent_part = path_parts[path_parts.len() - 2]; // 倒数第二部分

            // 查找同时包含这两部分的路径
            for (key, path) in &self.path_map {
                let key_parts: Vec<&str> = key.split('.').collect();

                // 查找键中是否包含目标部分和父部分
                let contains_target = key_parts.iter().any(|&part| part == target_part);
                let contains_parent = key_parts.iter().any(|&part| part == parent_part);

                if contains_target && contains_parent {
                    return Some(path.clone());
                }
            }

            // 只查找关键部分
            for (key, path) in &self.path_map {
                if key.contains(target_part) {
                    return Some(path.clone());
                }
            }
        }

        None
    }
    
    // Helper methods for tests
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

/// 递归处理源码映射树节点，构建DataModel路径到本地文件路径的映射
fn process_node(
    node: &RojoSourceMapNode,
    parent_path: &str,
    path_map: &mut HashMap<String, String>,
) {
    // 构建当前节点的完整路径
    let current_path = if parent_path.is_empty() {
        node.name.clone()
    } else {
        format!("{}.{}", parent_path, node.name)
    };

    // 如果节点有文件路径，添加到映射
    if !node.filePaths.is_empty() {
        path_map.insert(current_path.clone(), node.filePaths[0].clone());
    }

    // 递归处理子节点
    for child in &node.children {
        process_node(child, &current_path, path_map);
    }
}

/// 获取 Rojo Sourcemap
pub fn get_sourcemap(
    command: &str,
    config_dir: Option<&str>,
) -> Result<RojoSourceMap, Box<dyn Error>> {
    

    // 创建一个默认的空sourcemap作为fallback
    let empty_sourcemap = r#"{"entries":{}}"#;

    // 尝试几种不同的方法执行命令
    let mut sourcemap_json = String::new();
    let mut success = false;

    // 方法1: 使用powershell.exe
    if !success {
        let mut cmd = Command::new("powershell.exe");
        cmd.args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            command,
        ]);

        if let Some(dir) = config_dir {
            cmd.current_dir(dir);
        }

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    sourcemap_json = String::from_utf8_lossy(&output.stdout).to_string();
                    success = true;
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    eprintln!("powershell.exe 方法失败: {}", error);
                }
            }
            Err(e) => {
                eprintln!("powershell.exe 执行错误: {}", e);
            }
        }
    }

    // 方法2: 使用cmd.exe /c powershell
    if !success {
        let cmd_str = format!(
            "powershell.exe -NoProfile -ExecutionPolicy Bypass -Command \"{}\"",
            command.replace("\"", "\\\"")
        );
        eprintln!("尝试cmd.exe方法: {}", cmd_str);

        let mut cmd = Command::new("cmd.exe");
        cmd.args(["/c", &cmd_str]);

        if let Some(dir) = config_dir {
            cmd.current_dir(dir);
        }

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    sourcemap_json = String::from_utf8_lossy(&output.stdout).to_string();
                    success = true;
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    eprintln!("cmd.exe 方法失败: {}", error);
                }
            }
            Err(e) => {
                eprintln!("cmd.exe 执行错误: {}", e);
            }
        }
    }

    // 如果命令执行失败，尝试寻找缓存的sourcemap文件
    if !success {
        eprintln!("尝试查找缓存的sourcemap文件");
        let cache_paths = [
            "sourcemap.json",
            ".robloxrc/sourcemap.json",
            ".rojo/sourcemap.json",
        ];

        let base_dir = config_dir.unwrap_or(".");

        for path in cache_paths.iter() {
            let full_path = std::path::Path::new(base_dir).join(path);
            eprintln!("检查缓存文件: {:?}", full_path);

            if let Ok(content) = std::fs::read_to_string(&full_path) {
                sourcemap_json = content;
                success = true;
                eprintln!("找到缓存的sourcemap: {:?}", full_path);
                break;
            }
        }
    }

    // 如果所有方法都失败，使用空的sourcemap
    if !success {
        eprintln!("所有方法都失败，使用空的sourcemap");
        sourcemap_json = empty_sourcemap.to_string();
    }

    // 移除可能存在的非JSON输出前缀
    let json_start = sourcemap_json.find('{').unwrap_or(0);
    let clean_json = &sourcemap_json[json_start..];

    // 尝试解析JSON
    match RojoSourceMap::new(clean_json) {
        Ok(map) => Ok(map),
        Err(e) => {
            eprintln!("解析sourcemap失败: {}", e);
            // 如果JSON解析失败，使用空的sourcemap
            RojoSourceMap::new(empty_sourcemap)
        }
    }
}

/// 将 Roblox 堆栈跟踪行转换为本地文件路径
pub fn convert_stack_trace(line: &str, sourcemap: &RojoSourceMap) -> String {
    // 示例: ServerScriptService.TS.Runtime.__tests__.os-lib.jack:58
    let re = Regex::new(r"(.+?):(\d+)").unwrap();

    if let Some(captures) = re.captures(line) {
        let datamodel_path = captures.get(1).unwrap().as_str();
        let line_number = captures.get(2).unwrap().as_str();

        // 尝试在sourcemap中查找对应的本地路径
        if let Some(file_path) = sourcemap.get_file_path(datamodel_path) {
            return format!("{}:{}", file_path, line_number);
        } else {
            // 额外尝试特殊处理
            // 检查是否包含特殊字符@
            if datamodel_path.contains('@') {
                let simplified_path = datamodel_path.replace("@", "");

                // 尝试在路径映射中查找简化后的路径
                for (key, path) in sourcemap.get_path_map() {
                    let simplified_key = key.replace("@", "");

                    // 如果简化后的键包含简化后的路径
                    if simplified_key.contains(&simplified_path) {
                        return format!("{}:{}", path, line_number);
                    }
                }

                // 尝试按路径部分匹配
                let parts: Vec<&str> = datamodel_path.split('.').collect();
                if !parts.is_empty() {
                    let last_part = parts[parts.len() - 1];

                    // 查找包含最后一部分的路径
                    for (key, path) in sourcemap.get_path_map() {
                        if key.contains(last_part) {
                            return format!("{}:{}", path, line_number);
                        }
                    }
                }
            }
        }
    }

    // 如果找不到对应路径，返回原始行
    line.to_string()
}

/// 转换多行堆栈信息
pub fn convert_stack_trace_text(text: &str, sourcemap: &RojoSourceMap) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut result = Vec::with_capacity(lines.len());

    for line in lines {
        let converted_line = convert_stack_trace(line, sourcemap);
        result.push(converted_line);
    }

    result.join("\n")
}


