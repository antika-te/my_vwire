mod ads_filter;
mod config;

use anyhow::{Context, Result};
use aya::{Bpf, programs::{Xdp, XdpFlags}};
use clap::Parser;
use log::{info, warn, error};
use std::sync::Arc;
use tokio::sync::RwLock;

use ads_filter::AdsFilterEngine;
use config::Config;

/// Vwire 廣告過濾系統 - Userspace 控制程序
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 網路接口名稱 (例如：eth0)
    #[arg(short, long, default_value = "eth0")]
    interface: String,

    /// 配置文件路徑
    #[arg(short, long)]
    config: Option<String>,

    /// 啟用調試日誌
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日誌
    let args = Args::parse();
    if args.debug {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("debug")
        ).init();
    } else {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("info")
        ).init();
    }

    info!("🚀 啟動 Vwire 廣告過濾系統");
    info!("  網路接口：{}", args.interface);

    // 加載配置文件
    let config = load_config(args.config.as_deref())?;
    
    // 初始化廣告過濾引擎
    let filter_engine = Arc::new(RwLock::new(
        AdsFilterEngine::new(&config)?
    ));

    info!("📡 廣告過濾系統已就緒");
    info!("  提示：目前為 PoC 版本，僅做規則匹配演示");
    info!("  下一步：實現 AF_XDP 數據通道和 HTTP 流量劫持");
    
    // 啟動測試循環
    run_demo_loop(filter_engine).await?;

    Ok(())
}

fn load_config(path: Option<&str>) -> Result<Config> {
    match path {
        Some(config_path) => {
            info!("  配置文件：{}", config_path);
            // TODO: 從文件加載配置
            Ok(Config::default())
        },
        None => {
            info!("  使用默認配置");
            Ok(Config::default())
        }
    }
}

async fn run_demo_loop(filter_engine: Arc<RwLock<AdsFilterEngine>>) -> Result<()> {
    info!("");
    info!("📝 演示模式：測試廣告規則匹配");
    info!("  輸入 URL 進行測試 (輸入 q 退出)");
    info!("");

    let stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut buffer = String::new();
    let mut reader = std::pin::pin!(stdin);

    loop {
        print!("URL> ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();
        
        buffer.clear();
        let result = tokio::io::AsyncBufReadExt::read_line(
            &mut reader, 
            &mut buffer
        ).await?;
        
        if result == 0 {
            break; // EOF
        }
        
        let input = buffer.trim();
        
        if input == "q" || input == "quit" {
            break;
        }
        
        if input.is_empty() {
            continue;
        }
        
        // 解析 URL (簡單處理)
        let (host, path) = parse_url(input);
        
        // 檢查是否為廣告
        let mut engine = filter_engine.write().await;
        let is_ad = engine.is_advertisement(&host, &path);
        
        if is_ad {
            println!("  ❌ 廣告 - 已攔截");
        } else {
            println!("  ✓ 正常內容 - 放行");
        }
        
        // 每 10 次請求打印統計
        if engine.get_stats().total_requests % 10 == 0 {
            engine.print_stats();
        }
    }

    // 打印最終統計
    info!("");
    filter_engine.read().await.print_stats();
    
    Ok(())
}

fn parse_url(url: &str) -> (String, String) {
    // 簡單解析：提取 host 和 path
    // 生產環境應該使用 url crate
    
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    
    if let Some(slash_pos) = url.find('/') {
        let host = url[..slash_pos].to_string();
        let path = url[slash_pos..].to_string();
        (host, path)
    } else {
        (url.to_string(), "/".to_string())
    }
}
