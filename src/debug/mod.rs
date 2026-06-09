use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::non_blocking::WorkerGuard;

pub fn init_log() -> WorkerGuard {
    // 创建一个名为 debug.log 的文件，放在当前目录下
    let file_appender = tracing_appender::rolling::never(".", "debug.log");

    // 创建非阻塞写入器 (非常重要，防止异步阻塞)
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(fmt::layer()
            .with_ansi(false) // 禁用颜色代码，防止 log 文件乱码
            .with_writer(non_blocking)) // 写入到文件
        .init();

    // 返回 guard，必须在 main 函数中持有它，否则日志系统会被立即 drop
    guard
}