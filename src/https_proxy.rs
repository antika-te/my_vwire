//! HTTPS MITM 代理 (完整版)
//! 
//! 使用 rcgen 0.13 + rustls 0.23 + tokio-rustls

use anyhow::Result;
use log::{info, debug, warn, error};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use rcgen::Certificate;

/// HTTPS MITM 代理
pub struct HttpsProxy {
    /// CA 證書 DER
    ca_cert: Vec<u8>,
    /// CA 私鑰 DER
    ca_key: Vec<u8>,
}

impl HttpsProxy {
    /// 創建 HTTPS 代理
    pub fn new() -> Result<Self> {
        info!("  初始化 HTTPS MITM 代理...");
        
        // 生成 CA 證書和私鑰
        let (ca_cert, ca_key) = Self::generate_ca()?;
        
        info!("  ✓ HTTPS 代理初始化完成");
        info!("  ✓ CA 證書已生成");
        
        Ok(HttpsProxy {
            ca_cert,
            ca_key,
        })
    }

    /// 生成自簽 CA 證書
    fn generate_ca() -> Result<(Vec<u8>, Vec<u8>)> {
        use rcgen::{CertificateParams, DistinguishedName, IsCa, KeyUsagePurpose};
        
        let mut params = CertificateParams::default();
        
        // 設置 CA 屬性
        let mut dn = DistinguishedName::new();
        dn.push(rcgen::DnType::CommonName, "Vwire Ad Filter CA");
        dn.push(rcgen::DnType::OrganizationName, "Vwire Project");
        params.distinguished_name = dn;
        
        // 設置為 CA 證書
        use rcgen::BasicConstraints;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
        ];
        
        // 生成證書
        let key_pair = rcgen::KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;
        
        info!("  ✓ CA 證書生成成功 (P-256 ECDSA)");
        
        Ok((
            cert.der().to_vec(),
            key_pair.serialized_der().to_vec(),
        ))
    }

    /// 獲取 CA 證書 PEM 格式
    pub fn get_ca_cert_pem(&self) -> String {
        use base64::{Engine, engine::general_purpose::STANDARD};
        
        let pem_content = STANDARD.encode(&self.ca_cert);
        
        // 每 64 個字符換行
        let lines: Vec<String> = pem_content
            .chars()
            .collect::<Vec<_>>()
            .chunks(64)
            .map(|c| c.iter().collect::<String>())
            .collect();
        
        format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
            lines.join("\n")
        )
    }

    /// 導出 CA 證書到文件
    pub fn export_ca_cert(&self, path: &str) -> Result<()> {
        use std::fs::write;
        let pem = self.get_ca_cert_pem();
        write(path, pem.as_bytes())?;
        info!("  ✓ CA 證書已導出到 {}", path);
        Ok(())
    }

    /// 檢查 HTTPS 是否啟用
    pub fn is_enabled(&self) -> bool {
        true
    }

    /// 處理 HTTPS 請求 (完整 TLS 解密)
    pub async fn handle_https_request(
        &self,
        client_stream: &mut TcpStream,
        hostname: &str,
    ) -> Result<()> {
        debug!("處理 HTTPS 請求：{}", hostname);
        
        // 1. 生成服務器證書
        let server_cert = self.generate_server_cert(hostname)?;
        let server_key = rcgen::KeyPair::from_der(&self.ca_key)?;
        
        // 2. 配置 TLS 服務器
        use rustls::ServerConfig;
        use tokio_rustls::TlsAcceptor;
        
        let mut server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![rustls::Certificate(server_cert)],
                rustls::PrivateKey(server_key.serialized_der().to_vec()),
            )?;
        
        server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        
        let acceptor = TlsAcceptor::from(Arc::new(server_config));
        
        // 3. TLS Handshake with client
        let mut tls_stream = acceptor.accept(client_stream.try_clone()?).await?;
        debug!("TLS handshake 完成：{}", hostname);
        
        // 4. 讀取 HTTPS 請求
        let mut buffer = vec![0u8; 8192];
        let n = tls_stream.read(&mut buffer).await?;
        buffer.truncate(n);
        
        // 5. 檢查是否包含廣告
        let request_str = String::from_utf8_lossy(&buffer);
        if self.is_ad_request(&request_str, hostname) {
            debug!("攔截廣告請求：{}", hostname);
            // 返回 403 或 204
            let block_response = b"HTTP/1.1 204 No Content\r\n\r\n";
            tls_stream.write_all(block_response).await?;
            return Ok(());
        }
        
        // 6. 轉發到真實服務器
        self.forward_to_upstream(&mut tls_stream, hostname, &buffer).await?;
        
        Ok(())
    }
    
    /// 檢查是否為廣告請求
    fn is_ad_request(&self, request: &str, hostname: &str) -> bool {
        // 檢查域名黑名單
        let ad_domains = [
            "googleadservices.com",
            "doubleclick.net",
            "ads.facebook.com",
            "ads.twitter.com",
            "amazon-adsystem.com",
            "criteo.com",
            "adnxs.com",
        ];
        
        if ad_domains.iter().any(|d| hostname.contains(d)) {
            return true;
        }
        
        // 檢查關鍵字
        let ad_keywords = ["ads", "adserver", "analytics", "tracking"];
        ad_keywords.iter().any(|k| request.to_lowercase().contains(k))
    }
    
    /// 轉發到上游服務器
    async fn forward_to_upstream(
        &self,
        tls_stream: &mut tokio_rustls::server::TlsStream<TcpStream>,
        hostname: &str,
        request: &[u8],
    ) -> Result<()> {
        // 連接到真實服務器
        let upstream = TcpStream::connect(format!("{}:443", hostname)).await?;
        
        // TLS 連接到上游
        use rustls::ClientConfig;
        use tokio_rustls::TlsConnector;
        
        let client_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(rustls::RootCertStore::empty())
            .with_no_client_auth();
        
        let connector = TlsConnector::from(Arc::new(client_config));
        let mut upstream_tls = connector.connect(hostname.try_into()?, upstream).await?;
        
        // 轉發請求
        upstream_tls.write_all(request).await?;
        
        // 讀取響應並返回給客戶端
        let mut buffer = vec![0u8; 4096];
        loop {
            let n = upstream_tls.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            tls_stream.write_all(&buffer[..n]).await?;
        }
        
        Ok(())
    }
}

impl Default for HttpsProxy {
    fn default() -> Self {
        Self::new().expect("Failed to create HttpsProxy")
    }
}

/// 安裝 CA 證書指南
pub fn print_ca_install_guide() {
    println!("\n📋 安裝 CA 證書指南");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("要在您的設備上信任 Vwire CA 證書，請執行以下步驟：");
    println!();
    println!("1. 導出 CA 證書:");
    println!("   ./vwire-filter --export-ca-cert");
    println!("   或手動：./vwire-filter --export-ca-cert=/path/to/vwire-ca.crt");
    println!();
    println!("2. 安裝到系統信任存儲:");
    println!();
    println!("   Ubuntu/Debian:");
    println!("   sudo cp vwire-ca.crt /usr/local/share/ca-certificates/");
    println!("   sudo update-ca-certificates");
    println!();
    println!("   CentOS/RHEL:");
    println!("   sudo cp vwire-ca.crt /etc/pki/ca-trust/source/anchors/");
    println!("   sudo update-ca-trust");
    println!();
    println!("   macOS:");
    println!("   sudo security add-trusted-cert vwire-ca.crt");
    println!();
    println!("   Windows:");
    println!("   雙擊 vwire-ca.crt → 安裝證書 → 本地計算機");
    println!("   選擇「將所有證書放入以下存儲」→ 瀏覽 → 受信任的根證書頒發機構");
    println!();
    println!("3. 對於 Android/iOS 設備:");
    println!("   將證書文件傳輸到設備並通過設置安裝");
    println!();
    println!("⚠️  注意：");
    println!("   - 僅在受信任的網絡環境中使用");
    println!("   - 不要在公共網絡或未授權的設備上安裝");
    println!("   - 此 CA 證書僅用於廣告過濾目的");
    println!("   - 安裝後設備會完全信任 Vwire 過濾器，請謹慎使用");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ca() {
        let proxy = HttpsProxy::new().unwrap();
        assert!(!proxy.ca_cert.is_empty());
        assert!(!proxy.ca_key.is_empty());
    }

    #[test]
    fn test_get_ca_cert_pem() {
        let proxy = HttpsProxy::new().unwrap();
        let pem = proxy.get_ca_cert_pem();
        assert!(pem.contains("BEGIN CERTIFICATE"));
        assert!(pem.contains("END CERTIFICATE"));
    }
}
