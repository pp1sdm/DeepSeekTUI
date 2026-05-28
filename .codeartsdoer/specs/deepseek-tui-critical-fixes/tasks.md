# DeepSeekTUI 关键缺陷修复 - 编码任务列表

## 1. 环境变量初始化优化（dotenv 单次初始化）
- [ ] 在 `src/main.rs` 的 `main()` 函数中，于 `debug::init_log()` 之前添加 `dotenvy::dotenv().ok();` 调用，确保 `.env` 文件仅在应用启动时加载一次
- [ ] 验收标准：`main()` 函数中存在且仅存在一处 `dotenvy::dotenv()` 调用；`run()` 函数中无 `dotenv` 调用

## 2. 移除 llm 模块中的 dotenv 调用
- [ ] 在 `src/llm/mod.rs` 中，删除 `use dotenvy::dotenv;` 导入语句和 `dotenv().ok();` 调用行
- [ ] 验收标准：`src/llm/mod.rs` 中不再包含任何 `dotenv` 相关代码

## 3. 修正 API 模型名称
- [ ] 在 `src/llm/mod.rs` 中，将请求体 JSON 的 `"model": "deepseek-v4-flash"` 修改为 `"model": "deepseek-chat"`
- [ ] 验收标准：API 请求中 model 字段值为 `deepseek-chat`，符合 DeepSeek API 官方支持的合法模型名称

## 4. 增强 llm 模块错误信息可读性
- [ ] 在 `src/llm/mod.rs` 中，将 `env::var("DEEPSEEK_KEY")?` 改为 `env::var("DEEPSEEK_KEY").map_err(|_| anyhow!("API 密钥未配置，请检查 DEEPSEEK_KEY 环境变量"))?`，提供面向用户的可读提示
- [ ] 在 `src/llm/mod.rs` 中，确认 HTTP 错误响应的错误消息格式为 `HTTP {status}: {body摘要}`（当前已实现，确认保留）
- [ ] 在 `src/llm/mod.rs` 中，将 JSON 解析失败的错误消息由 `anyhow!("响应格式错误")` 改为 `anyhow!("响应格式错误: 缺少 choices[0].message.content")`，提供更具体的字段路径信息
- [ ] 验收标准：API 密钥缺失时返回"API 密钥未配置"提示；HTTP 错误包含状态码；JSON 解析失败包含字段路径

## 5. 新增 ApiResult 枚举和 App 异步通信字段
- [ ] 在 `src/app/mod.rs` 中，定义 `ApiResult` 枚举：`pub enum ApiResult { Ok(String), Err(String) }`
- [ ] 在 `src/app/mod.rs` 中，为 `App` 结构体新增 `is_loading: bool` 字段（初始值 `false`）
- [ ] 在 `src/app/mod.rs` 中，为 `App` 结构体新增 `tx: Option<tokio::sync::mpsc::UnboundedSender<ApiResult>>` 字段（初始值 `None`）
- [ ] 在 `src/app/mod.rs` 中，更新 `App::new()` 初始化逻辑，设置 `is_loading: false` 和 `tx: None`
- [ ] 添加必要的 `use` 导入语句（`tokio::sync::mpsc`）
- [ ] 验收标准：`App` 结构体包含 `is_loading` 和 `tx` 字段；`ApiResult` 枚举已定义且可在模块外部使用

## 6. App 新增 set_sender 和 handle_api_result 方法
- [ ] 在 `src/app/mod.rs` 中，实现 `pub fn set_sender(&mut self, tx: mpsc::UnboundedSender<ApiResult>)` 方法，将发送端存入 `self.tx`
- [ ] 在 `src/app/mod.rs` 中，实现 `pub fn handle_api_result(&mut self, result: ApiResult)` 方法：
  - 将 `self.is_loading` 设为 `false`
  - 匹配 `ApiResult::Ok(reply)` 时调用 `self.session.add_assistant_message(reply)`
  - 匹配 `ApiResult::Err(err_msg)` 时调用 `self.session.add_assistant_message(format!("[错误] {}", err_msg))`
- [ ] 验收标准：`set_sender` 正确设置 channel 发送端；`handle_api_result` 正确处理成功和失败结果，加载状态恢复为 false

## 7. 新增 format_error 辅助函数
- [ ] 在 `src/app/mod.rs` 中，实现 `fn format_error(err: &Box<dyn std::error::Error + Send + Sync>) -> String` 辅助函数：
  - 检测超时错误 → 返回 `"请求超时（60秒），请检查网络连接后重试"`
  - 检测连接错误 → 返回 `"网络连接失败: {错误描述}"`
  - 检测 HTTP 错误（包含状态码） → 返回原始错误描述
  - 检测 API 密钥错误 → 返回 `"API 密钥未配置，请检查 DEEPSEEK_KEY 环境变量"`
  - 其他 → 返回原始错误描述 `err.to_string()`
- [ ] 验收标准：各类型错误均能转换为面向用户的可读中文提示

## 8. 重构 apply_action 中 SendMessage 为异步任务
- [ ] 在 `src/app/mod.rs` 中，将 `apply_action` 签名由 `pub async fn` 改为 `pub fn`（SendMessage 不再 await）
- [ ] 重写 `Action::SendMessage` 分支：
  - 获取输入框文本，若 `trim()` 为空则提前返回
  - 调用 `self.session.add_user_message(text)` 添加用户消息
  - 设置 `self.is_loading = true`
  - 清空输入框 `self.textarea = TextArea::new(vec![])`
  - 克隆 `self.tx`，若 `Some(tx)` 则 `tokio::spawn` 异步任务：
    - 克隆 `messages = self.session.messages.clone()`
    - 在 spawn 闭包中调用 `agent::Agent.run(&messages).await`
    - 成功时通过 `tx.send(ApiResult::Ok(reply))` 发送结果
    - 失败时通过 `tx.send(ApiResult::Err(format_error(&e)))` 发送错误
- [ ] 验收标准：SendMessage 不阻塞主事件循环；API 调用在独立异步任务中执行；主循环保持响应

## 9. 主事件循环增加 channel 创建和轮询
- [ ] 在 `src/main.rs` 的 `run()` 函数开头，创建 `let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<app::ApiResult>();`
- [ ] 调用 `app.set_sender(tx)` 将发送端传入 App
- [ ] 在事件循环 `while` 迭代中，于 `terminal.draw()` 之前增加 channel 非阻塞轮询：
  - `while let Ok(result) = rx.try_recv() { app.handle_api_result(result); }`
- [ ] 将 `app.apply_action(action).await` 改为 `app.apply_action(action)`（不再需要 await）
- [ ] 将 `run()` 函数签名由 `async fn` 改为 `fn`（不再需要 async，但保持 tokio runtime 可用）
- [ ] 验收标准：主循环每 100ms 轮询 channel 结果；API 响应到达后在下一轮询周期处理；界面保持响应

## 10. UI 渲染支持加载指示
- [ ] 在 `src/ui/mod.rs` 的 `render_session_list` 函数中，在构建 `items` 列表之后、创建 `List` 之前，增加逻辑：当 `app.is_loading` 为 `true` 时，向 `items` 追加一个 `ListItem::new(Line::from(Span::styled("🤖 思考中...", Style::default().italic())))`
- [ ] 验收标准：API 请求期间会话列表末尾显示斜体"思考中..."提示；API 响应到达后提示消失

## 11. UI 错误消息红色样式渲染
- [ ] 在 `src/ui/mod.rs` 的 `render_session_list` 函数中，修改消息列表项渲染逻辑：对于角色为 `assistant` 且内容以 `[错误]` 开头的消息，使用红色前景色（`Color::Red`）渲染；其余消息保持原样式
- [ ] 验收标准：错误消息在会话列表中以红色文本显示，便于用户快速识别

## 12. 防止重复提交和并发安全
- [ ] 在 `src/app/mod.rs` 的 `Action::SendMessage` 分支中，当 `self.is_loading` 为 `true` 时提前返回，防止用户在 API 请求期间重复提交
- [ ] 验收标准：请求进行中时用户按 Ctrl+Enter 不会触发新的 API 调用

## 13. 编译验证与基本功能测试
- [ ] 执行 `cargo check` 确认所有修改后代码无编译错误
- [ ] 执行 `cargo build` 确认项目可正常构建
- [ ] 验证应用启动正常：运行应用确认终端界面可正常显示
- [ ] 验证模型名称修正：确认 API 请求体中 model 字段为 `deepseek-chat`
- [ ] 验证异步化：提交消息后界面保持响应，会话列表显示"思考中..."，API 响应后正确渲染回复
- [ ] 验证错误处理：配置错误的 API 密钥后提交消息，确认应用不崩溃且会话列表显示红色错误提示
- [ ] 验证 dotenv 单次初始化：确认 `.env` 仅在启动时加载一次
