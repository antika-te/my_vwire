use crate::config::Config;
use anyhow::Result;
use aho_corasick::AhoCorasick;
use log::{info, debug};

/// 廣告規則匹配引擎
/// 使用 Aho-Corasick 自動機算法實現高效多模式匹配
pub struct AdsFilterEngine {
    /// Aho-Corasick 自動機實例
    ac_automaton: AhoCorasick,
    
    /// 白名單 (允許的域名)
    whitelist: Vec<String>,
    
    /// 統計信息
    stats: FilterStats,
}

#[derive(Default)]
pub struct FilterStats {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub allowed_requests: u64,
}

impl AdsFilterEngine {
    /// 創建新的過濾引擎
    pub fn new(config: &Config) -> Result<Self> {
        info!("🔧 初始化廣告過濾引擎...");
        
        // 構建規則列表
        let patterns = Self::build_patterns();
        
        // 構建 Aho-Corasick 自動機
        let ac = AhoCorasick::new(patterns)
            .expect("Failed to build Aho-Corasick automaton");
        
        let mut engine = AdsFilterEngine {
            ac_automaton: ac,
            whitelist: Self::build_whitelist(),
            stats: FilterStats::default(),
        };
        
        info!("  ✓ 廣告過濾引擎初始化完成");
        info!("  ✓ Aho-Corasick 自動機建成功");
        info!("  ✓ 已加載 {} 條白名單規則", engine.whitelist.len());
        
        Ok(engine)
    }
    
    /// 構建匹配模式列表
    fn build_patterns() -> Vec<String> {
        let mut patterns = vec![
            // 常見廣告域名
            "doubleclick.net".to_string(),
            "googleadservices.com".to_string(),
            "adservice.google.com".to_string(),
            
            // 廣告路徑特徵
            "/ads/".to_string(),
            "/get_ads".to_string(),
            "/banner?".to_string(),
            "/tracking?".to_string(),
            "/pixel?".to_string(),
            
            // 廣告相關關鍵字
            "adservice".to_string(),
            "adserver".to_string(),
            "adtracker".to_string(),
            "analytics".to_string(),
        ];
        
        info!("  已加載 {} 條廣告規則", patterns.len());
        patterns
    }
    
    /// 構建白名單
    fn build_whitelist() -> Vec<String> {
        vec![
            "google.com".to_string(),
            "youtube.com".to_string(),
            "netflix.com".to_string(),
            "github.com".to_string(),
            "stackoverflow.com".to_string(),
        ]
    }
    
    /// 檢查請求是否為廣告
    pub fn is_advertisement(&mut self, host: &str, path: &str) -> bool {
        self.stats.total_requests += 1;
        
        // 1. 檢查白名單
        if self.is_whitelisted(host) {
            self.stats.allowed_requests += 1;
            debug!("  [白名單放行] {} {}", host, path);
            return false;
        }
        
        // 2. 使用 Aho-Corasick 自動機匹配
        let full_url = format!("{}{}", host, path);
        if self.ac_automaton.is_match(&full_url) {
            self.stats.blocked_requests += 1;
            debug!("  [規則匹配攔截] {} {}", host, path);
            return true;
        }
        
        self.stats.allowed_requests += 1;
        false
    }
    
    fn is_whitelisted(&self, host: &str) -> bool {
        self.whitelist.iter().any(|domain| host.contains(domain))
    }
    
    /// 獲取統計信息
    pub fn get_stats(&self) -> &FilterStats {
        &self.stats
    }
    
    /// 打印統計信息
    pub fn print_stats(&self) {
        info!("📊 過濾統計:");
        info!("  總請求數：{}", self.stats.total_requests);
        info!("  攔截廣告：{}", self.stats.blocked_requests);
        info!("  放行請求：{}", self.stats.allowed_requests);
        
        if self.stats.total_requests > 0 {
            let block_rate = (self.stats.blocked_requests as f64 / self.stats.total_requests as f64) * 100.0;
            info!("  攔截率：{:.2}%", block_rate);
        }
    }
}
