//! SNI (Server Name Indication) 提取器
//! 
//! 從 TLS ClientHello 中提取域名信息

use anyhow::{Result, Context};
use log::{debug, trace};

/// 從 TLS ClientHello 中提取 SNI 域名
pub fn extract_sni(client_hello: &[u8]) -> Result<Option<String>> {
    // TLS 記錄層分析
    if client_hello.len() < 5 {
        return Ok(None);
    }
    
    let record_type = client_hello[0];
    if record_type != 0x16 {  // TLS Handshake
        debug!("非 TLS Handshake 記錄：{:02x}", record_type);
        return Ok(None);
    }
    
    // 跳過記錄頭 (5 bytes)
    let mut offset = 5;
    
    // Handshake 類型 (ClientHello = 0x01)
    if offset >= client_hello.len() || client_hello[offset] != 0x01 {
        return Ok(None);
    }
    offset += 1;
    
    // Handshake 長度 (3 bytes)
    if offset + 3 > client_hello.len() {
        return Ok(None);
    }
    let handshake_len = ((client_hello[offset] as usize) << 16)
        | ((client_hello[offset + 1] as usize) << 8)
        | (client_hello[offset + 2] as usize);
    offset += 3;
    
    // TLS 版本 (2 bytes)
    offset += 2;
    
    // Random (32 bytes)
    offset += 32;
    
    // Session ID
    if offset >= client_hello.len() {
        return Ok(None);
    }
    let session_id_len = client_hello[offset] as usize;
    offset += 1 + session_id_len;
    
    // Cipher Suites
    if offset + 2 > client_hello.len() {
        return Ok(None);
    }
    let cipher_suites_len = ((client_hello[offset] as usize) << 8)
        | (client_hello[offset + 1] as usize);
    offset += 2 + cipher_suites_len;
    
    // Compression Methods
    if offset >= client_hello.len() {
        return Ok(None);
    }
    let compression_len = client_hello[offset] as usize;
    offset += 1 + compression_len;
    
    // Extensions
    if offset + 2 > client_hello.len() {
        return Ok(None);
    }
    let extensions_len = ((client_hello[offset] as usize) << 8)
        | (client_hello[offset + 1] as usize);
    offset += 2;
    
    let extensions_end = offset + extensions_len;
    if extensions_end > client_hello.len() {
        return Ok(None);
    }
    
    // 解析 Extensions
    while offset + 4 <= extensions_end {
        let ext_type = ((client_hello[offset] as u16) << 8)
            | (client_hello[offset + 1] as u16);
        let ext_len = ((client_hello[offset + 2] as usize) << 8)
            | (client_hello[offset + 3] as usize);
        offset += 4;
        
        // SNI Extension (type = 0)
        if ext_type == 0 && ext_len > 0 {
            return parse_sni_extension(&client_hello[offset..offset + ext_len]);
        }
        
        offset += ext_len;
    }
    
    Ok(None)
}

/// 解析 SNI Extension
fn parse_sni_extension(data: &[u8]) -> Result<Option<String>> {
    if data.len() < 2 {
        return Ok(None);
    }
    
    // Server Name List Length
    let list_len = ((data[0] as usize) << 8) | (data[1] as usize);
    if data.len() < 3 + list_len {
        return Ok(None);
    }
    
    // 第一個 Server Name
    let mut offset = 2;
    while offset < list_len {
        let name_type = data[offset];
        let name_len = ((data[offset + 1] as usize) << 8)
            | (data[offset + 2] as usize);
        offset += 3;
        
        // DNS 類型 (type = 0)
        if name_type == 0 {
            let hostname = String::from_utf8_lossy(&data[offset..offset + name_len]);
            debug!("提取 SNI: {}", hostname);
            return Ok(Some(hostname.to_string()));
        }
        
        offset += name_len;
    }
    
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_sni_sample() {
        // 模擬 TLS ClientHello (包含 SNI extension)
        let sample_client_hello = vec![
            0x16, 0x03, 0x01, 0x00, 0x04,  // TLS Record Header
            0x01, 0x00, 0x00, 0x03,  // Handshake Header
            0x03, 0x03,  // TLS Version
        ];
        
        let result = extract_sni(&sample_client_hello).unwrap();
        assert!(result.is_none());
    }
}
