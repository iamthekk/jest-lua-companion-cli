# jest-lua Companion CLI

致谢：

- [jackTabsCode](https://github.com/jackTabsCode/testez-companion-cli)


## 功能

- 通过命令行运行, 命令Roblox Studio运行测试
- 在命令行打印测试结果
- 将测试结果中的堆栈信息, 转化为本地路径

## 配置

在项目根目录下创建 `jest-lua-companion.config.json` 文件：

```json
{
  "roots": ["ServerScriptService/TS/Runtime"],
  "runCLI-options": {
    "ci": true,
    "clearMocks": true,
    "testNamePattern": "test.*",
    "testMatch": ["**/*.spec.lua"],
    "verbose": true
  },
  "rojoSourceMapCommand": "rojo sourcemap"
}
```

- roots: 待测试的 DataModel 路径,以 `/` 分割. 必填
- runCLI-options: 默认为空
  - [文档](https://jsdotlua.github.io/jest-lua/cli), 数组以 `|` 分割
  - 特例,SetupFiles: 在[littensy](https://github.com/littensy/charm-example)的[例子](https://github.com/littensy/charm-example/blob/main/test/spec.server.luau)中, 将 `SetupFiles` 对象也放到了 `runCLI-options` 中, 而在[官方文档](https://jsdotlua.github.io/jest-lua/configuration)中, 没有获得明确信息.
  - 此处我选择为[例子](https://github.com/littensy/charm-example/blob/main/test/spec.server.luau)中的`SetupFile`提供默认支持,硬编码到了插件内.
  - 如果你配置更多的`SetupFiles`, 请在`runCLI-options`中配置以 `|`分割的DataModel路径, 同 `roots` 配置方式.
- rojoSourceMapCommand: 用于获取 `rojo sourcemap` 的 stdout 的指令. 默认为空.

## 安装

**注意: 仅支持windows**

**安装包**

```sh
npm install jest-lua-companion-cli
```

**安装插件**

复制 `Plugin/jest-roblox-companion-plugin.server.lua` 到 `roblox` 插件目录.

## 用法

**查看帮助**

```bash
npx jest-lua-companion-cli -h
```

**运行测试**

```bash
npx jest-lua-companion-cli
```

**cli选项**

支持所有 jest-lua 的 [CLI 选项](https://jsdotlua.github.io/jest-lua/cli#testmatch-arraystring), 比如:

```bash
npx jest-lua-companion-cli -t "testName"
```

只需在终端运行 `jest-lua-companion-cli`。如果有多个 Place 打开，会提示你选择一个。随后会输出测试结果！

你也可以将其加入 pre-commit 钩子，在提交前确保测试通过（当然，需要你的 Place 已在 Studio 中打开）。

## 开发

### 构建和运行

```bash
# 构建项目
cargo build

# 运行CLI
cargo run
```

### 运行单元测试

项目包含完整的单元测试套件，覆盖了核心功能模块：

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test --lib config
cargo test --lib rojo_sourcemap
cargo test --lib jest_results

# 查看测试输出
cargo test -- --nocapture
```

### 测试覆盖范围

单元测试覆盖以下模块：

- `config.rs`: 配置解析和序列化
- `rojo_sourcemap.rs`: Rojo源码映射处理和堆栈跟踪转换
- `jest_results.rs`: Jest测试结果解析和格式化

## 其他

- 本地服务和roblox插件指向: `http://127.0.0.1:28860`