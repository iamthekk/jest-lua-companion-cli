#!/usr/bin/env node

/*
    运行本程序后, 将指定的源文件复制到 {user}/AppData/Local/Roblox/Plugins 目录中, 覆盖同名文件
    用法: node roblox-plugin-deploy.js <源文件路径> <目标文件名>
*/

const fs = require('fs');
const path = require('path');

// 解析命令行参数
const args = process.argv.slice(2);

// 显示帮助信息
function showHelp() {
    console.log(`
用法: node roblox-plugin-deploy.js <源文件路径> <目标文件名>

参数:
  <源文件路径>    要复制的源文件路径（相对或绝对路径）
  <目标文件名>    复制到 Roblox Plugins 目录中的文件名

选项:
  -h, --help     显示此帮助信息

示例:
  node roblox-plugin-deploy.js ../src/AssestProxy/LocalAssetProxy.lua LocalAssetProxy.lua
    `);
}

// 检查帮助选项
if (args.includes('-h') || args.includes('--help')) {
    showHelp();
    process.exit(0);
}

// 检查参数数量
if (args.length !== 2) {
    console.error('错误: 需要提供 2 个参数');
    showHelp();
    process.exit(1);
}

const [sourcePath, targetFileName] = args;

// 检查源文件是否存在
const pluginSrcPath = path.resolve(sourcePath);
if (!fs.existsSync(pluginSrcPath)) {
    console.error(`错误: 源文件不存在: ${pluginSrcPath}`);
    process.exit(1);
}

// 创建目标目录
const userPath = path.join(process.env.LOCALAPPDATA, 'Roblox/Plugins');
if (!fs.existsSync(userPath)) {
    fs.mkdirSync(userPath, { recursive: true });
}

const pluginTargetPath = path.join(userPath, targetFileName);

// 复制文件
try {
    fs.copyFileSync(pluginSrcPath, pluginTargetPath);
    console.log('copied ' + pluginTargetPath);
} catch (error) {
    console.error(`错误: 复制文件失败: ${error.message}`);
    process.exit(1);
}
