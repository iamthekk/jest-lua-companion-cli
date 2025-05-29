#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { getStudiotPath } = require('./roblox-studio-path.js');

/**
 * 部署插件到Roblox插件目录
 * @param {string} source 源文件路径
 * @param {string} target 目标文件名
 */
function deployPlugin(source, target) {
    console.log('开始部署插件...');
    
    // 检查源文件是否存在
    const pluginSrcPath = path.resolve(source);
    if (!fs.existsSync(pluginSrcPath)) {
        console.error(`错误: 源文件不存在: ${pluginSrcPath}`);
        return false;
    }

    // 创建目标目录
    const userPath = path.join(process.env.LOCALAPPDATA, 'Roblox/Plugins');
    if (!fs.existsSync(userPath)) {
        fs.mkdirSync(userPath, { recursive: true });
    }

    const pluginTargetPath = path.join(userPath, target);

    // 复制文件
    try {
        fs.copyFileSync(pluginSrcPath, pluginTargetPath);
        console.log(`插件已复制到: ${pluginTargetPath}`);
        return true;
    } catch (error) {
        console.error(`错误: 复制插件失败: ${error.message}`);
        return false;
    }
}

/**
 * 修改Roblox Studio的FFlag设置
 */
function modifyFFlag() {
    console.log('开始修改FFlag设置...');
    
    const studioPath = getStudiotPath();
    if (!studioPath) {
        console.error('错误: 无法获取 Roblox Studio 路径');
        return false;
    }

    const clientSettingsPath = path.join(studioPath, 'ClientSettings', 'ClientAppSettings.json');
    
    try {
        let settings = {};
        
        // 如果文件存在，读取现有设置
        if (fs.existsSync(clientSettingsPath)) {
            const content = fs.readFileSync(clientSettingsPath, 'utf8');
            settings = JSON.parse(content);
        } else {
            // 确保目录存在
            const clientSettingsDir = path.dirname(clientSettingsPath);
            if (!fs.existsSync(clientSettingsDir)) {
                fs.mkdirSync(clientSettingsDir, { recursive: true });
            }
        }

        // 修改FFlag设置
        settings.FFlagEnableLoadModule = true;

        // 写入文件
        fs.writeFileSync(clientSettingsPath, JSON.stringify(settings, null, 2));
        console.log(`FFlag已修改: ${clientSettingsPath}`);
        console.log('FFlagEnableLoadModule 已设置为 true');
        return true;
    } catch (error) {
        console.error(`错误: 修改FFlag失败: ${error.message}`);
        return false;
    }
}

/**
 * 主函数
 */
function main() {
    console.log('jest-lua-companion-cli 初始化工具');
    console.log('================================');
    
    // 获取插件文件的绝对路径
    const pluginSourcePath = path.join(__dirname, '..', 'plugin', 'jest-lua-companion-plugin.server.lua');
    
    // 部署插件
    const pluginDeployed = deployPlugin(
        pluginSourcePath,
        'jest-lua-companion-plugin.server.lua'
    );
    
    // 修改FFlag
    const fflagModified = modifyFFlag();
    
    console.log('\n初始化结果:');
    console.log(`插件部署: ${pluginDeployed ? '成功' : '失败'}`);
    console.log(`FFlag修改: ${fflagModified ? '成功' : '失败'}`);
    
    if (pluginDeployed && fflagModified) {
        console.log('\n✅ 初始化完成！现在可以使用 jest-lua-companion-cli 运行测试了。');
    } else {
        console.log('\n❌ 初始化过程中出现错误，请检查上述错误信息。');
        process.exit(1);
    }
}

// 如果作为主模块运行，则执行main函数
if (require.main === module) {
    main();
}

module.exports = {
    deployPlugin,
    modifyFFlag
}; 