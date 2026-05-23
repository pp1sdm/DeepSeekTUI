use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

fn main() {
    println!("=== crossterm 最小测试 ===");
    println!("尝试 enable_raw_mode...");
    
    match enable_raw_mode() {
        Ok(()) => {
            println!("成功！现在 disable...");
            if let Err(e) = disable_raw_mode() {
                println!("disable 失败: {}", e);
            } else {
                println!("全部正常！crossterm 可用。");
            }
        }
        Err(e) => {
            println!("enable_raw_mode 失败: {}", e);
            println!("");
            println!("这是系统级问题，可能原因：");
            println!("1. 杀毒软件/安全软件拦截了控制台 API");
            println!("2. Windows 11 的'终端'默认设置导致 ConPTY 冲突");
            println!("3. 编译目标不是标准 Windows 控制台子系统");
            println!("");
            println!("尝试解决：");
            println!("- 以管理员身份运行 CMD/PowerShell，再 cargo run");
            println!("- 检查是否安装了 360、火绒等软件，临时关闭后重试");
            println!("- 在 Windows 设置 -> 隐私和安全性 -> Windows 安全中心 -> 应用和浏览器控制 -> 基于声誉的保护 -> 关闭'检查应用和文件'");
        }
    }
}