const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

/**
 * 从 Windows 注册表获取值
 * @param {string} key 注册表键
 * @param {string} valueName 值名称
 * @returns {string|null} 注册表值或 null
 */
function getRegistryValue(key, valueName) {
    try {
        if (process.platform !== 'win32') {
            return null;
        }

        // 使用 /reg:64 确保在64位系统上正确读取，并使用 chcp 65001 设置UTF-8编码
        const command = `chcp 65001 >nul && reg query "${key}" /v "${valueName}" /reg:64`;
        const output = execSync(command, { encoding: 'utf8' });

        // 解析输出以获取值
        const lines = output.split('\n');
        for (const line of lines) {
            const parts = line.trim().split(/\s{4,}/);  // 使用正则表达式匹配4个或更多空格
            if (parts.length >= 3 && parts[0] === valueName) {
                return parts[parts.length - 1];
            }
        }

        return null;
    } catch (error) {
        console.error(`获取注册表值时出错: ${error.message}`);
        return null;
    }
}

/**
 * 获取 Roblox Studio  目录
 * @returns {string|null}  目录路径或 null
 */
function getStudiotPath() {
    try {

        if (process.platform === 'win32') {
            // 从注册表获取 Studio 的 ContentFolder
            const contentFolder = getRegistryValue(
                'HKEY_CURRENT_USER\\SOFTWARE\\Roblox\\RobloxStudio',
                'ContentFolder'
            );

            if (contentFolder && fs.existsSync(contentFolder)) {
                var studioFolder = contentFolder.replace(/[\\/]content[\\/]?$/, '');
                return studioFolder;
            }
        }

        return null;
    } catch (error) {
        console.error(`获取 Studio content 目录时出错: ${error.message}`);
        return null;
    }
}



module.exports = {
    getStudiotPath,
}; 