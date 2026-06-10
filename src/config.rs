/// 系統配置
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// HTTP 端口 (默认 80)
    pub http_port: u16,
    
    /// HTTPS 端口 (默认 443，後續擴展用)
    pub https_port: u16,
    
    /// 是否啟用 HTTPS MITM
    pub enable_https_mitm: bool,
    
    /// 規則列表更新間隔 (秒)
    pub rules_update_interval: u64,
    
    /// 白名單 IP 列表 (這些 IP 的流量直接 bypass)
    pub bypass_ips: Vec<String>,
}

impl Config {
    pub fn default() -> Self {
        Config {
            http_port: 80,
            https_port: 443,
            enable_https_mitm: false,
            rules_update_interval: 3600, // 每小時更新一次規則
            bypass_ips: vec![],
        }
    }
}
