use crate::blockchain::Blockchain;
use colored::*;
use std::io::{self, Write};

/// 工具函数模块，提供各种辅助功能

/// 显示程序横幅
pub fn display_banner() {
    println!("{}", "
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║    🦀 Rust 简单区块链实现 🔗                                ║
║                                                              ║
║    一个用于学习区块链原理的简单实现                          ║
║    包含工作量证明、SHA-256哈希、JSON持久化                   ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
    ".bright_cyan());
}

/// 显示主菜单
pub fn display_main_menu() {
    println!("\n{}", "📋 ===== 主菜单 =====".bright_yellow());
    println!("1. 📦 挖掘新区块");
    println!("2. 📊 显示区块链");
    println!("3. ✅ 验证区块链");
    println!("4. 💾 保存区块链");
    println!("5. 📂 加载区块链");
    println!("6. ⚙️  设置难度");
    println!("7. 📈 显示统计信息");
    println!("8. 🚀 批量挖矿");
    println!("9. 🔍 查看区块详情");
    println!("0. 👋 退出程序");
    print!("\n请选择操作 (0-9): ");
    //stdout()刷新缓冲区 flush()确保输出立即显示 unwrap()处理可能的错误
    io::stdout().flush().unwrap();
}

/// 获取用户输入
pub fn get_user_input() -> String {
    let mut input = String::new();
    // stdin()获取标准输入 read_line()读取一行输入 expect()处理可能的错误
    io::stdin().read_line(&mut input).expect("读取输入失败");
    //trim()去除前后的空白字符 to_string()转换为String
    input.trim().to_string()
}

/// 获取用户输入的数字
pub fn get_number_input(prompt: &str) -> Option<u32> {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let input = get_user_input();
    //parse()将字符串转换为数字 ok()处理转换失败的情况
    input.parse().ok()
}

/// 获取用户输入的字符串（非空）
pub fn get_string_input(prompt: &str) -> String {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        
        let input = get_user_input();
        //trim()去除前后的空白字符 is_empty()判断是否为空
        if !input.trim().is_empty() {
            return input;
        }
        println!("{}", "输入不能为空，请重新输入。".red());
    }
}

/// 显示加载动画
pub fn show_loading(message: &str, duration_ms: u64) {
    let chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let start = std::time::Instant::now();
    //while循环持续动画 duration_ms指定动画持续时间
    while start.elapsed().as_millis() < duration_ms as u128 {
        for &c in &chars {
            print!("\r{} {}", c, message);
            io::stdout().flush().unwrap();
            //thread::sleep()暂停线程 sleep()指定暂停时间
            std::thread::sleep(std::time::Duration::from_millis(100));
            //判断动画是否超时
            if start.elapsed().as_millis() >= duration_ms as u128 {
                break;
            }
        }
    }
    println!("\r✅ {}", message);
}

/// 显示成功消息
pub fn show_success(message: &str) {
    println!("{} {}", "✅".green(), message.green());
}

/// 显示错误消息
pub fn show_error(message: &str) {
    println!("{} {}", "❌".red(), message.red());
}

/// 显示警告消息
pub fn show_warning(message: &str) {
    println!("{} {}", "⚠️".yellow(), message.yellow());
}

/// 显示信息消息
pub fn show_info(message: &str) {
    println!("{} {}", "ℹ️".blue(), message.blue());
}

/// 格式化哈希值显示
pub fn format_hash(hash: &str, max_length: usize) -> String {
    if hash.len() <= max_length {
        hash.to_string()
    } else {
        let half = max_length / 2;
        format!("{}...{}", &hash[..half], &hash[hash.len()-(max_length-half-3)..])
    }
}

/// 格式化文件大小
pub fn format_file_size(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// 格式化时间间隔
pub fn format_duration(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.0} 毫秒", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{:.2} 秒", seconds)
    } else if seconds < 3600.0 {
        format!("{:.1} 分钟", seconds / 60.0)
    } else {
        format!("{:.1} 小时", seconds / 3600.0)
    }
}

/// 显示区块链统计信息的美化版本
pub fn display_pretty_stats(blockchain: &Blockchain) {
    let stats = blockchain.get_statistics();
    
    println!("\n{}", "📊 ===== 区块链统计 =====".bright_yellow());
    println!("🔗 区块总数: {}", stats.total_blocks.to_string().bright_cyan());
    println!("📦 链大小: {}", format_file_size(stats.total_size).bright_cyan());
    println!("💳 交易数量: {}", stats.total_transactions.to_string().bright_cyan());
    println!("🎯 当前难度: {}", stats.current_difficulty.to_string().bright_cyan());
    println!("💰 挖矿奖励: {}", stats.mining_reward.to_string().bright_cyan());
    println!("⏱️  平均出块: {}", format_duration(stats.average_block_time).bright_cyan());
    
    // 计算一些附加统计
    if stats.total_blocks > 1 {
        let latest_block = blockchain.get_latest_block();
        let genesis_block = &blockchain.chain[0];
        let chain_duration = latest_block.timestamp - genesis_block.timestamp;
        
        println!("📅 链时长: {}", format_duration(chain_duration.num_seconds() as f64).bright_cyan());
        println!("🚀 平均哈希率: {} H/s", stats.average_hash_rate.to_string().bright_cyan());
        println!("💎 总尝试次数: {}", stats.total_attempts.to_string().bright_cyan());
    }
    
    println!("\n{}", "---".repeat(20).bright_yellow());
}