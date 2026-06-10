mod ads_filter;
mod config;
mod http_parser;
mod xdp_socket;

use anyhow::{Context, Result};
use aya::BpfLoader;
use aya::programs::{Xdp, XdpFlags};
use clap::Parser;
use log::{info, warn, error};
use std::sync::Arc;
use tokio::sync::{RwLock, Notify};

use ads_filter::AdsFilterEngine;
use config::Config;
use http_parser::HttpParser;

/// Vwire 廣告過濾系統 - v0.2.0
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

    /// 演示模式 (不實際攔截)
    #[arg(long, default_value = "false")]
    demo: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日誌
    let args = Args::parse();
    if args.debug {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("vwire_filter=debug,info")
        ).init();
    } else {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("vwire_filter=info")
        ).init();
    }

    info!("🚀 啟動 Vwire 廣告過濾系統 v0.2.0");
    info!("  網路接口：{}", args.interface);
    info!("  運行模式：{}", if args.demo { "演示模式" } else { "生產模式" });

    // 加載配置文件
    let config = load_config(args.config.as_deref())?;
    
    // 初始化組件
    let filter_engine = Arc::new(RwLock::new(AdsFilterEngine::new(&config)?));
    let _http_parser = Arc::new(HttpParser::new());
    
    // 加載並附加 eBPF 程序
    load_and_attach_ebpf(&args.interface)?;
    
    info!("📡 所有組件初始化完成");
    info!("  ✓ 廣告過濾引擎");
    info!("  ✓ HTTP 解析器");
    info!("  ✓ XDP 程序 (接口：{})", args.interface);

    // 啟動 Watchdog
    let shutdown_notify = Arc::new(Notify::new());
    let watchdog_handle = tokio::spawn(run_watchdog(shutdown_notify.clone()));

    // 啟動流量處理循環
    if args.demo {
        info!("");
        info!("📝 演示模式：測試規則匹配");
        info!("  輸入 URL 進行測試 (輸入 q 退出)");
        info!("");
        run_demo_loop(filter_engine).await?;
    } else {
        info!("");
        info!("📡 開始監聽並過濾 HTTP 流量...");
        
        // 嘗試創建 AF_XDP socket
        match xdp_socket::XdpSocket::new(&args.interface) {
            Ok(Some(_socket)) => {
                info!("  ✓ AF_XDP socket 初始化成功");
                warn!("⚠️  AF_XDP 流量處理邏輯開發中...");
                // TODO: 啟動實際流量處理
            }
            Ok(None) => {
                warn!("⚠️  AF_XDP socket 不可用，使用降級模式");
            }
            Err(e) => {
                warn!("⚠️  AF_XDP socket 初始化失敗：{}", e);
            }
        }
    }

    // 清理
    shutdown_notify.notify_one();
    let _ = watchdog_handle.await;

    info!("👋 再見！");
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

fn load_and_attach_ebpf(interface: &str) -> Result<()> {
    info!("  加載 eBPF 程序...");
    
    // 嘗試加載編譯好的 eBPF binary
    let mut bpf = BpfLoader::new()
        .load_file("ebpf/vwire-filter-ebpf")
        .or_else(|_| BpfLoader::new().load_file("/usr/share/vwire/vwire-filter-ebpf"))
        .context("無法加載 eBPF 程序，請先編譯 ebpf/vwire-filter-ebpf")?;
    
    info!("  ✓ eBPF 程序加載成功");
    
    // 附加 XDP 程序
    info!("  附加 XDP 程序到接口 {}...", interface);
    
    let xdp_prog: &mut Xdp = bpf
        .program_mut("vwire_xdp")
        .context("找不到 XDP 程序")?
        .try_into()?;
    
    xdp_prog
        .load()
        .context("XDP 程序加載失敗")?;
    
    xdp_prog
        .attach(interface, XdpFlags::default())
        .context(format!("XDP 程序附加到接口 {} 失敗", interface))?;
    
    info!("  ✓ XDP 程序附加成功");
    Ok(())
}

async fn run_demo_loop(filter_engine: Arc<RwLock<AdsFilterEngine>>) -> Result<()> {
    use tokio::io::AsyncBufReadExt;
    
    let stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut buffer = String::new();
    let mut reader = std::pin::pin!(stdin);

    loop {
        print!("URL> ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();
        
        buffer.clear();
        let result = reader.read_line(&mut buffer).await?;
        
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
        
        // 解析 URL
        let (host, path) = http_parser::parse_url(input);
        
        // 檢查是否為廣告
        let mut engine = filter_engine.write().await;
        let is_ad = engine.is_advertisement(&host, &path);
        
        if is_ad {
            println!("  ❌ 廣告 - 已攔截 (HTTP 204)");
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

async fn run_watchdog(shutdown: Arc<Notify>) {
    info!("🐕 Watchdog 進程已啟動");
    
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
    let mut healthy = true;
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // 定期检查系统健康状态
                if !healthy {
                    warn!("Watchdog: 系統恢復健康");
                    healthy = true;
                }
            }
            _ = shutdown.notified() => {
                info!("Watchdog: 收到關閉信號，正在清理...");
                
                // TODO: 切換到透傳模式
                // 設置 BYPASS_ALL = 1
                
                break;
            }
        }
    }
    
    info!("Watchdog: 已停止");
}
