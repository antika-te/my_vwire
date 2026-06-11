//! HTTPS MITM 代理 (簡化版)
//! 
//! 使用 rcgen 0.13 + rustls 0.23

use anyhow::Result;
use log::{info, debug, warn};

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

    /// 處理 HTTPS 請求 (框架)
    pub async fn handle_https_request(
        &self,
        _client_stream: &mut tokio::net::TcpStream,
        _hostname: &str,
    ) -> Result<()> {
        // TODO: 完整的 HTTPS 處理流程
        debug!("HTTPS 處理流程開發中...");
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
