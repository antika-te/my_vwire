//! HTTPS MITM 代理
//! 
//! 實現透明 HTTPS 解密和內容過濾
//! 使用動態證書生成技術

use anyhow::{Context, Result};
use log::{info, debug, warn};
use rcgen::{Certificate, CertificateParams, DistinguishedName, KeyPair, IsCa, BasicConstraints, KeyUsagePurpose, ExtendedKeyUsagePurpose, PKCS_ECDSA_P256_SHA256};
use std::sync::Arc;

/// HTTPS MITM 代理
pub struct HttpsProxy {
    /// CA 證書
    ca_cert: Arc<Certificate>,
    /// CA 私鑰
    ca_key: Arc<KeyPair>,
}

impl HttpsProxy {
    /// 創建 HTTPS 代理
    pub fn new() -> Result<Self> {
        info!("  初始化 HTTPS MITM 代理...");
        
        // 生成自簽 CA 證書
        let (ca_cert, ca_key) = Self::generate_ca()?;
        let ca_cert = Arc::new(ca_cert);
        let ca_key = Arc::new(ca_key);
        
        info!("  ✓ HTTPS 代理初始化完成");
        info!("  ✓ CA 證書已生成");
        
        Ok(HttpsProxy {
            ca_cert,
            ca_key,
        })
    }

    /// 生成自簽 CA 證書
    fn generate_ca() -> Result<(Certificate, KeyPair)> {
        let mut params = CertificateParams::default();
        
        // 設置 CA 屬性
        let mut dn = DistinguishedName::new();
        dn.push(rcgen::DnType::CommonName, "Vwire Ad Filter CA");
        dn.push(rcgen::DnType::OrganizationName, "Vwire Project");
        params.distinguished_name = dn;
        
        // 設置為 CA 證書
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
        ];
        params.alg = &PKCS_ECDSA_P256_SHA256;
        
        // 生成證書
        let key_pair = KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;
        
        info!("  ✓ CA 證書生成成功");
        Ok((cert, key_pair))
    }

    /// 為目標域名生成動態證書
    pub fn generate_server_cert(&self, hostname: &str) -> Result<Certificate> {
        debug!("  為 {} 生成動態證書...", hostname);
        
        let mut params = CertificateParams::new(vec![hostname.to_string()]);
        
        // 設置證書屬性
        let mut dn = DistinguishedName::new();
        dn.push(rcgen::DnType::CommonName, hostname);
        dn.push(rcgen::DnType::OrganizationName, "Vwire Ad Filter");
        params.distinguished_name = dn;
        
        // 設置為终端實體證書
        params.is_ca = IsCa::NoCa;
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        params.extended_key_usages = vec![
            ExtendedKeyUsagePurpose::ServerAuth,
        ];
        params.alg = &PKCS_ECDSA_P256_SHA256;
        
        // 使用 CA 證書簽發
        let key_pair = KeyPair::generate()?;
        let cert = params.signed_by(&key_pair, &self.ca_cert, &self.ca_key)?;
        
        debug!("  ✓ 動態證書生成成功");
        
        Ok(cert)
    }

    /// 獲取 CA 證書 DER (用於安裝到客戶端信任存儲)
    pub fn get_ca_cert_der(&self) -> Vec<u8> {
        self.ca_cert.der().to_vec()
    }

    /// 獲取 CA 證書 PEM (用於展示)
    pub fn get_ca_cert_pem(&self) -> String {
        self.ca_cert.pem()
    }

    /// 獲取 HTTPS 代理狀態
    pub fn is_enabled(&self) -> bool {
        true
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
    println!();
    println!("2. 安裝到系統信任存儲:");
    println!();
    println!("   Ubuntu/Debian:");
    println!("   sudo cp vwire-ca.crt /usr/local/share/ca-certificates/");
    println!("   sudo update-ca-certificates");
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
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ca() {
        let (cert, key) = HttpsProxy::generate_ca().unwrap();
        assert!(!cert.der().is_empty());
        assert!(!key.serialized_der().is_empty());
    }

    #[test]
    fn test_generate_server_cert() {
        let proxy = HttpsProxy::new().unwrap();
        let cert = proxy.generate_server_cert("example.com").unwrap();
        assert!(!cert.der().is_empty());
    }
}
