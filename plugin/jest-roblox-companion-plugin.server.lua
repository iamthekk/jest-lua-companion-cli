task.spawn(function()
	-- 服务引用
	local RunService = game:GetService("RunService")
	local HttpService = game:GetService("HttpService")
	local LogService = game:GetService("LogService")

	-- 配置常量
	local IS_DEBUG = false -- 调试模式开关
	local BASE_URL = "http://127.0.0.1:28860" -- 本地服务器地址
	local POLLING_INTERVAL = 0.7 -- 轮询间隔(秒)

	-- 如果游戏正在运行中，则不执行插件逻辑
	if RunService:IsRunning() then
		return
	end

	-- 调试日志打印函数
	local function debugPrint(...)
		if IS_DEBUG then
			print(...)
		end
	end

	-- 日志函数封装
	local function log(printingFunction, message, ...)
		if IS_DEBUG then
			printingFunction("[Jest-Roblox Companion] " .. message, ...)
		end
	end

	-- 解析路径函数，将路径字符串转换为实例引用
	local function unwrapPath(path)
		local segments = string.split(path, "/")
		local lastParent = game

		for _, segment in ipairs(segments) do
			if not lastParent then
				return
			end
			lastParent = lastParent:FindFirstChild(segment)
		end

		return lastParent
	end

	-- 获取Jest运行器函数
	local function getJestRunner()
		-- 禁用颜色输出
		_G.NOCOLOR = true

		local ReplicatedStorage = game:GetService("ReplicatedStorage")
		local ServerScriptService = game:GetService("ServerScriptService")

		-- 引入Jest模块
		local Jest = require(ReplicatedStorage.rbxts_include.node_modules["@rbxts"].jest.src)

		-- 返回Jest运行函数
		return function(config, roots)
			-- 尝试获取ProcessService（用于测试完成后退出）
			local processServiceExists, ProcessService = pcall(function()
				return game:GetService("ProcessService")
			end)

			-- 设置核心清理脚本（每次测试前清理模块缓存）
			local coreSetupScript = [[
for key in _G do
	-- 清除已注册模块以重置roblox-ts运行时
	if typeof(key) == "Instance" and key:IsA("ModuleScript") then
		_G[key] = nil
	end
end

return nil
			]]

			-- 准备设置文件
			local setupFiles = {}

			-- 创建或刷新核心设置模块
			local coreModuleIns = game.ReplicatedStorage:FindFirstChild("__jest_setup__")
			if coreModuleIns then
				-- 每次必须传给jestCli新实例
				coreModuleIns:Destroy()
			end
			coreModuleIns = Instance.new("ModuleScript")
			coreModuleIns.Name = "__jest_setup__"
			coreModuleIns.Parent = game.ReplicatedStorage
			coreModuleIns.Source = coreSetupScript
			table.insert(setupFiles, coreModuleIns)

			-- 添加自定义设置文件
			if config.setupFiles then
				for k, v in config.setupFiles do
					local ins = unwrapPath(v)
					table.insert(setupFiles, ins)
				end
			end

			-- 更新配置中的设置文件
			config.setupFiles = setupFiles

			-- 运行Jest测试
			local status, result = Jest.runCLI(script, config, roots):awaitStatus()

			-- 处理测试结果和进程退出
			if status == "Rejected" then
				debugPrint(result)
			end

			-- 如果测试成功且没有失败的测试套件和测试用例，退出进程
			if
				status == "Resolved"
				and result.results.numFailedTestSuites == 0
				and result.results.numFailedTests == 0
			then
				if processServiceExists then
					ProcessService:ExitAsync(0)
				end
			end

			-- 如果有任何测试失败，以非零状态码退出
			if processServiceExists then
				ProcessService:ExitAsync(1)
			end

			return status, result
		end
	end

	-- 初始化标识信息
	local PlaceId = tostring(game.PlaceId)
	local PlaceName = game.Name
	local PlaceGUID = HttpService:GenerateGUID(false)

	debugPrint(
		"[调试] 初始化标识符 - PlaceId: " .. PlaceId .. ", PlaceName: " .. PlaceName .. ", GUID: " .. PlaceGUID
	)

	-- 请求头中的标识信息
	local identifierHeaders = {
		["place-id"] = PlaceId,
		["place-name"] = PlaceName,
		["place-guid"] = PlaceGUID,
	}

	-- 结果报告器对象
	local reporter = {
		report = function(results, caughtFromJest)
			debugPrint("[调试] 开始报告测试结果")
			debugPrint("[调试] 捕获自Jest错误: " .. tostring(caughtFromJest))

			local Headers = {
				["Content-Type"] = "application/json",
				["place-guid"] = PlaceGUID,
			}

			if caughtFromJest then
				Headers["caught-jest-error"] = "true"
			end

			debugPrint("[调试] 发送结果到服务器...")
			local ok, serverResponse = pcall(HttpService.RequestAsync, HttpService, {
				Url = BASE_URL .. "/results",
				Method = "POST",
				Headers = Headers,
				Body = HttpService:JSONEncode(results),
			})

			if not ok or serverResponse.StatusCode ~= 200 then
				log(warn, "无法向服务器报告测试结果 (" .. tostring(serverResponse) .. ")")
				debugPrint("[调试] 结果报告失败: " .. tostring(serverResponse))
			else
				debugPrint("[调试] 结果报告成功")
			end
		end,
	}

	-- 日志服务连接变量
	local logServiceConnection

	debugPrint("[调试] 插件初始化开始")
	debugPrint("[调试] 开始轮询循环")

	-- 主轮询循环
	while true do
		debugPrint("[调试] 发送轮询请求...")
		local ok, serverResponse = pcall(HttpService.RequestAsync, HttpService, {
			Url = BASE_URL .. "/poll",
			Method = "GET",
			Headers = identifierHeaders,
		})

		-- 处理有效的服务器响应
		if ok and serverResponse.StatusCode == 200 then
			debugPrint("[调试] 收到有效轮询响应，准备执行测试")
			local config = HttpService:JSONDecode(serverResponse.Body)
			local roots = {}

			-- 清除日志并设置日志捕获
			debugPrint("[调试] 清除日志并设置日志捕获")
			LogService:ClearOutput()
			logServiceConnection = LogService.MessageOut:Connect(function(message, messageType)
				if IS_DEBUG then
					debugPrint("[调试] 捕获日志: " .. message)
					pcall(HttpService.RequestAsync, HttpService, {
						Url = BASE_URL .. "/logs",
						Method = "POST",
						Body = HttpService:JSONEncode({
							message = message,
							messageType = messageType.Value,
						}),
						Headers = {
							["Content-Type"] = "application/json",
							["place-guid"] = PlaceGUID,
						},
					})
				end
			end)

			-- 动态加载Jest
			debugPrint("[调试] 尝试加载Jest包")
			local jestOk, jestRunner = pcall(function()
				return getJestRunner()
			end)

			-- 处理Jest加载失败
			if not jestOk then
				log(warn, "无法加载Jest包:")
				debugPrint(jestRunner) -- 这是错误消息
				debugPrint("[调试] Jest加载失败: " .. tostring(jestRunner))

				-- 报告错误
				debugPrint("[调试] 报告Jest加载错误")
				reporter.report({
					results = {
						success = false,
						numTotalTestSuites = 0,
						numTotalTests = 0,
						numPassedTests = 0,
						numFailedTests = 0,
						testResults = {},
						errors = { "无法加载Jest包: " .. tostring(jestRunner) },
					},
				}, true)

				-- 清理日志连接
				if logServiceConnection then
					logServiceConnection:Disconnect()
					logServiceConnection = nil
				end

				task.wait(POLLING_INTERVAL)
				continue
			end

			debugPrint("[调试] Jest加载成功")

			-- 创建Jest配置
			local jestConfig = config["runCLI-options"] or {}

			log(debugPrint, "开始执行Jest-Roblox测试...", config)

			-- 显示测试过滤条件
			if jestConfig.testNamePattern then
				log(debugPrint, "测试名称过滤: " .. jestConfig.testNamePattern)
			end

			if jestConfig.testPathPattern then
				log(debugPrint, "测试路径过滤: " .. jestConfig.testPathPattern)
			end

			-- 处理测试根目录
			local roots = {}
			if config.testRoots then
				for k, v in config.testRoots do
					local ins = unwrapPath(v)
					table.insert(roots, ins)
				end
			end

			-- 运行Jest测试
			debugPrint("[调试] 开始执行Jest测试")
			local testsOk, testResult = jestRunner(jestConfig, roots)

			-- 清理日志连接
			debugPrint("[调试] 清理日志连接")
			if logServiceConnection then
				logServiceConnection:Disconnect()
				logServiceConnection = nil
			end

			-- 处理测试结果
			if testsOk == true or testsOk == "Resolved" then
				debugPrint("[调试] Jest测试执行成功完成")
				reporter.report(testResult, false)
			else
				log(warn, "捕获到Jest-Roblox错误:")
				debugPrint(testResult)
				debugPrint("[调试] Jest测试执行出错: " .. tostring(testResult))

				-- 报告错误
				debugPrint("[调试] 报告Jest执行错误")
				reporter.report({
					results = {
						success = false,
						numTotalTestSuites = 0,
						numTotalTests = 0,
						numPassedTests = 0,
						numFailedTests = 0,
						testResults = {},
					},
				}, true)
			end
		elseif ok and serverResponse.StatusCode == 204 then
			-- 正常的空响应，继续轮询
			debugPrint("[调试] 轮询响应204，暂无测试任务")
		elseif ok then
			-- 其他非预期响应
			log(warn, "从服务器收到非预期的响应: " .. serverResponse.StatusCode)
			debugPrint("[调试] 非预期响应状态码: " .. serverResponse.StatusCode)
			if serverResponse.Body and serverResponse.Body ~= "" then
				log(warn, "响应体: " .. serverResponse.Body)
				debugPrint("[调试] 响应体: " .. serverResponse.Body)
			end
		else
			-- 连接错误，但不打印警告，因为服务器可能尚未启动
			debugPrint("[调试] 轮询连接失败，服务器可能未启动")
		end

		debugPrint("[调试] 等待下一轮轮询...")
		task.wait(POLLING_INTERVAL)
	end
end)
