//! 後端連接管理器
//! 
//! 負責連接到真實後端服務器並轉發請求
//! 支持 HTTP 和 HTTPS 協議

use anyhow::Result;
use log::{info, debug, error};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// 後端連接管理器
pub struct BackendConnection {
    /// 後端服务器地址
    host: String,
    /// 後端服务器端口
    port: u16,
    /// 連接超時 (秒)
    timeout_secs: u64,
}

impl BackendConnection {
    pub fn new(host: String, port: u16) -> Self {
        BackendConnection {
            host,
            port,
            timeout_secs: 30,
        }
    }

    /// 連接到後端並轉發請求
    pub async fn forward_request(&self, request: &[u8]) -> Result<Vec<u8>> {
        debug!("  連接後端：{}:{}", self.host, self.port);
        
        // 1. 連接到後端
        let stream = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            TcpStream::connect(format!("{}:{}", self.host, self.port))
        ).await??;
        
        let (mut read_half, mut write_half) = stream.into_split();
        
        // 2. 發送請求
        write_half.write_all(request).await?;
        write_half.flush().await?;
        debug!("  ✓ 請求已發送到後端");
        
        // 3. 接收響應
        let mut response = Vec::new();
        let mut buffer = vec![0u8; 8192];
        
        loop {
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                read_half.read(&mut buffer)
            ).await {
                Ok(Ok(0)) => break, // 連接關閉
                Ok(Ok(n)) => {
                    response.extend_from_slice(&buffer[..n]);
                }
                Ok(Err(e)) => {
                    error!("  讀取後端響應失敗：{}", e);
                    break;
                }
                Err(_) => {
                    debug!("  後端響應超時");
                    break;
                }
            }
        }
        
        debug!("  ✓ 收到後端響應 ({} bytes)", response.len());
        Ok(response)
    }
}

/// TLS 後端連接 (HTTPS)
pub struct TlsBackendConnection {
    backend: BackendConnection,
}

impl TlsBackendConnection {
    pub fn new(host: String, port: u16) -> Self {
        TlsBackendConnection {
            backend: BackendConnection::new(host, port),
        }
    }

    /// 通過 TLS 連接轉發請求
    pub async fn forward_request(&self, request: &[u8]) -> Result<Vec<u8>> {
        // TODO: 實現 TLS 連接
        // 這需要實際連接到後端的 HTTPS 服務
        // 目前暫時返回錯誤
        
        anyhow::bail!("TLS backend forwarding not yet implemented");
    }
}

/// 反向代理服务
pub struct ReverseProxy {
    /// 後端連接池 (簡化版，實際應使用連接池)
    backends: std::collections::HashMap<String, Arc<BackendConnection>>,
}

impl ReverseProxy {
    pub fn new() -> Self {
        ReverseProxy {
            backends: std::collections::HashMap::new(),
        }
    }

    /// 獲取或創建後端連接
    pub fn get_connection(&mut self, host: &str, port: u16) -> Arc<BackendConnection> {
        let key = format!("{}:{}", host, port);
        
        self.backends
            .entry(key)
            .or_insert_with(|| {
                Arc::new(BackendConnection::new(host.to_string(), port))
            })
            .clone()
    }

    /// 轉發請求到後端
    pub async fn forward(&mut self, host: &str, port: u16, request: &[u8]) -> Result<Vec<u8>> {
        let conn = self.get_connection(host, port);
        conn.forward_request(request).await
    }
}

impl Default for ReverseProxy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要真實的後端服务器
    async fn test_backend_connection() {
        let conn = BackendConnection::new("httpbin.org".to_string(), 80);
        let request = b"GET /get HTTP/1.1\r\nHost: httpbin.org\r\n\r\n";
        
        let response = conn.forward_request(&request[..]).await;
        assert!(response.is_ok());
        assert!(response.unwrap().starts_with(b"HTTP/1.1 200"));
    }
}
