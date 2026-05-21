mod agent;
mod app;
mod context;
mod llm;
mod session;

use session::{Session, message};
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    // Ctrl+C 退出
    ctrlc::set_handler(|| {
        println!("\n再见！");
        std::process::exit(0);
    })
    .expect("设置信号处理器失败");

    // 创建会话
    let mut sion = Session::new();

    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();

        // 拿到用户输入
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();

        // 去掉换行符和空格
        let input = buf.trim();

        let msg = message::Message::user(input);

        // 将用户输入给到上下文
        sion.add_message(msg);

        // 调 LLM
        let res = llm::chat(&sion.messages).await;

        match res {
            Ok(text) => {
                // 从 JSON 中提取 AI 回复的文本内容
                let content = text["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("无内容")
                    .to_string();

                // 给到用户
                println!("{}", content);

                // 加入会话历史
                let msg = message::Message::assistant(content);
                sion.add_message(msg);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}
