//! HTTP/HTTPS 流量處理器
//! 
//! 負責完整的流量處理流程：
//! 1. 從 AF_XDP socket 接收數據包
//! 2. 解析 HTTP/HTTPS 請求
//! 3. 匹配廣告規則
//! 4. 構造並發送響應 (204 攔截或轉發)

use crate::ads_filter::AdsFilterEngine;
use crate::http_parser::{HttpParser, HttpRequest};
use crate::backend::ReverseProxy;
use log::{info, debug, warn, error};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 流量處理結果
#[derive(Debug)]
pub enum ProcessResult {
    /// 已攔截 (返回 HTTP 204)
    Blocked(Vec<u8>),
    /// 需要轉發到後端
    Forward,
    /// 已轉發 (包含後端響應)
    Forwarded(Vec<u8>),
}

/// 流量處理器
pub struct TrafficProcessor {
    /// HTTP 解析器
    http_parser: HttpParser,
    
    /// 反向代理 (後端連接管理)
    reverse_proxy: ReverseProxy,
    
    /// 統計信息
    stats: TrafficStats,
}

#[derive(Debug, Default)]
pub struct TrafficStats {
    /// 總處理包數
    pub total_packets: u64,
    
    /// HTTP 請求數
    pub http_requests: u64,
    
    /// 攔截廣告數
    pub ads_blocked: u64,
    
    /// 放行請求數
    pub requests_allowed: u64,
    
    /// 處理錯誤數
    pub errors: u64,
}

impl TrafficProcessor {
    pub fn new() -> Self {
        TrafficProcessor {
            http_parser: HttpParser::new(),
            reverse_proxy: ReverseProxy::new(),
            stats: TrafficStats::default(),
        }
    }

    /// 處理單個 HTTP 請求
    /// 
    /// # Arguments
    /// * `data` - 原始 TCP payload
    /// * `filter_engine` - 廣告過濾引擎
    /// 
    /// # Returns
    /// * `ProcessResult` - 處理結果 (攔截/轉發/錯誤)
    pub async fn process_http_request(
        &mut self,
        data: &[u8],
        filter_engine: Arc<RwLock<AdsFilterEngine>>,
    ) -> ProcessResult {
        self.stats.total_packets += 1;
        
        // 1. 解析 HTTP 請求
        let request = match self.http_parser.parse_request(data) {
            Some(req) => {
                self.stats.http_requests += 1;
                req
            }
            None => {
                debug!("  無法解析 HTTP 請求 ({} bytes)", data.len());
                self.stats.errors += 1;
                return ProcessResult::Forward; // 無法解析，需要轉發
            }
        };
        
        debug!("  HTTP 請求：{} {} {}", request.method, request.host, request.path);
        
        // 2. 匹配廣告規則
        let is_ad = {
            let mut engine = filter_engine.write().await;
            engine.is_advertisement(&request.host, &request.path)
        };
        
        if is_ad {
            // 3a. 是廣告 - 返回 204 攔截
            self.stats.ads_blocked += 1;
            debug!("  ❌ 廣告已攔截：{}{}", request.host, request.path);
            
            ProcessResult::Blocked(HttpParser::build_204_response())
        } else {
            // 3b. 正常內容 - 轉發到後端
            self.stats.requests_allowed += 1;
            debug!("  ✓ 放行：{}{}", request.host, request.path);
            
            ProcessResult::Forward
        }
    }

    /// 處理完整請求並獲取響應
    pub async fn handle_request(
        &mut self,
        client_data: &[u8],
        filter_engine: Arc<RwLock<AdsFilterEngine>>,
    ) -> Result<ProcessResult, std::io::Error> {
        match self.process_http_request(client_data, filter_engine).await {
            ProcessResult::Blocked(response) => {
                // 已攔截，直接返回響應
                Ok(ProcessResult::Blocked(response))
            }
            ProcessResult::Forward => {
                // 需要解析出 host 和 port
                if let Some(request) = self.http_parser.parse_request(client_data) {
                    let port = if request.path.starts_with("https") { 443 } else { 80 };
                    let host = request.host.split(':').next().unwrap_or(&request.host).to_string();
                    
                    // 轉發到後端
                    match self.reverse_proxy.forward(&host, port, client_data).await {
                        Ok(response) => Ok(ProcessResult::Forwarded(response)),
                        Err(e) => {
                            error!("  轉發失敗：{}", e);
                            self.stats.errors += 1;
                            Ok(ProcessResult::Blocked(HttpParser::build_502_response()))
                        }
                    }
                } else {
                    Ok(ProcessResult::Forward)
                }
            }
            ProcessResult::Forwarded(_) => unreachable!(),
        }
    }

    /// 獲取統計信息
    pub fn get_stats(&self) -> &TrafficStats {
        &self.stats
    }

    /// 打印統計信息
    pub fn print_stats(&self) {
        info!("📊 流量處理統計:");
        info!("  總包數：{}", self.stats.total_packets);
        info!("  HTTP 請求：{}", self.stats.http_requests);
        info!("  攔截廣告：{}", self.stats.ads_blocked);
        info!("  放行請求：{}", self.stats.requests_allowed);
        info!("  處理錯誤：{}", self.stats.errors);
        
        if self.stats.http_requests > 0 {
            let block_rate = (self.stats.ads_blocked as f64 / self.stats.http_requests as f64) * 100.0;
            info!("  廣告攔截率：{:.2}%", block_rate);
        }
    }
}

impl Default for TrafficProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// 後端連接管理器 (TODO: 待實現)
pub struct BackendConnection {
    /// 後端服务器地址
    _host: String,
    /// 後端服务器端口
    _port: u16,
}

impl BackendConnection {
    pub fn new(host: String, port: u16) -> Self {
        BackendConnection {
            _host: host,
            _port: port,
        }
    }

    /// 連接到後端並轉發請求
    pub async fn forward_request(&self, _request: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        // TODO: 實現後端連接和轉發
        // 1. 連接到真實後端 (tokio::net::TcpStream::connect)
        // 2. 發送請求
        // 3. 接收響應
        // 4. 返回給客戶端
        
        todo!("Backend forwarding not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_advertisement_request() {
        let mut processor = TrafficProcessor::new();
        let config = crate::config::Config::default();
        let filter_engine = Arc::new(RwLock::new(
            crate::ads_filter::AdsFilterEngine::new(&config).unwrap()
        ));
        
        // 模擬廣告請求
        let request = b"GET /ads/banner.jpg HTTP/1.1\r\nHost: adserver.com\r\n\r\n";
        
        let result = processor.process_http_request(
            &request[..],
            filter_engine,
        ).await;
        
        match result {
            ProcessResult::Blocked(response) => {
                assert!(response.starts_with(b"HTTP/1.1 204"));
            }
            _ => panic!("Expected Blocked result"),
        }
    }

    #[tokio::test]
    async fn test_process_normal_request() {
        let mut processor = TrafficProcessor::new();
        let config = crate::config::Config::default();
        let filter_engine = Arc::new(RwLock::new(
            crate::ads_filter::AdsFilterEngine::new(&config).unwrap()
        ));
        
        // 模擬正常請求
        let request = b"GET /index.html HTTP/1.1\r\nHost: github.com\r\n\r\n";
        
        let result = processor.process_http_request(
            &request[..],
            filter_engine,
        ).await;
        
        match result {
            ProcessResult::Forward => {}, // 正常內容需要轉發
            _ => panic!("Expected Forward result"),
        }
    }
}
