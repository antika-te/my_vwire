use log::debug;

/// HTTP 請求解析器
/// 
/// 負責從原始 TCP payload 中提取：
/// - HTTP_METHOD (GET/POST 等)
/// - Host header
/// - URL path
/// - Content headers
pub struct HttpParser {
    /// 最大支持的 header 大小
    max_header_size: usize,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub host: String,
    pub headers: Vec<(String, String)>,
    pub body_offset: usize,
}

impl HttpParser {
    pub fn new() -> Self {
        HttpParser {
            max_header_size: 8192, // 8KB
        }
    }

    /// 解析 HTTP 請求
    /// 
    /// 從原始字節中解析 HTTP 請求
    /// 返回 HttpRequest 結構
    pub fn parse_request(&self, data: &[u8]) -> Option<HttpRequest> {
        if data.len() < 16 {
            return None;
        }

        // 找到 HTTP header 結束位置 (\r\n\r\n)
        let header_end = find_sequence(data, b"\r\n\r\n")?;
        
        // 解析請求行 (Request Line)
        let request_line_end = find_sequence(&data[..header_end], b"\r\n")?;
        let request_line = &data[..request_line_end];
        
        // 解析 METHOD PATH HTTP/1.x
        let (method, path) = self.parse_request_line(request_line)?;
        
        // 解析 headers
        let headers = self.parse_headers(&data[request_line_end + 2..header_end])?;
        
        // 提取 Host header
        let host = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("host"))
            .map(|(_, value)| value.clone())
            .unwrap_or_default();
        
        // 計算 body 起始位置
        let body_offset = header_end + 4;
        
        Some(HttpRequest {
            method,
            path,
            host,
            headers,
            body_offset,
        })
    }

    /// 解析請求行
    fn parse_request_line(&self, line: &[u8]) -> Option<(String, String)> {
        // 格式：METHOD PATH HTTP/1.x
        let parts: Vec<&[u8]> = line
            .split(|&b| b == b' ')
            .filter(|s| !s.is_empty())
            .collect();
        
        if parts.len() < 2 {
            return None;
        }

        let method = String::from_utf8_lossy(parts[0]).to_string();
        let path = String::from_utf8_lossy(parts[1]).to_string();
        
        // 驗證 method
        if !is_valid_method(&method) {
            return None;
        }

        Some((method, path))
    }

    /// 解析 headers
    fn parse_headers(&self, header_data: &[u8]) -> Option<Vec<(String, String)>> {
        let mut headers = Vec::new();
        
        let mut start = 0;
        while start < header_data.len() {
            // 找到下一行
            if let Some(end) = find_sequence(&header_data[start..], b"\r\n") {
                let line = &header_data[start..start + end];
                
                // 解析 header: value
                if let Some(colon_pos) = find_sequence(line, b":") {
                    let name = String::from_utf8_lossy(&line[..colon_pos]).trim().to_string();
                    let value = String::from_utf8_lossy(&line[colon_pos + 1..]).trim().to_string();
                    
                    if !name.is_empty() {
                        headers.push((name, value));
                    }
                }
                
                start += end + 2;
            } else {
                break;
            }
        }
        
        Some(headers)
    }

    /// 構造 HTTP 204 響應
    pub fn build_204_response() -> Vec<u8> {
        let response = "HTTP/1.1 204 No Content\r\n";
        let headers = [
            "Content-Length: 0",
            "Connection: close",
            "Cache-Control: no-store",
            "X-Ad-Filter: blocked",
        ].join("\r\n");
        
        format!("{}\r\n\r\n", response.to_string() + &headers)
            .into_bytes()
    }

    /// 構造 HTTP 404 響應 (可選)
    pub fn build_404_response() -> Vec<u8> {
        let response = "HTTP/1.1 404 Not Found\r\n";
        let headers = [
            "Content-Type: text/html",
            "Content-Length: 0",
            "Connection: close",
            "X-Ad-Filter: blocked",
        ].join("\r\n");
        
        format!("{}\r\n\r\n", response.to_string() + &headers)
            .into_bytes()
    }

    /// 構造 HTTP 502 響應 (後端錯誤)
    pub fn build_502_response() -> Vec<u8> {
        let response = "HTTP/1.1 502 Bad Gateway\r\n";
        let headers = [
            "Content-Type: text/html",
            "Content-Length: 0",
            "Connection: close",
            "X-Ad-Filter: error",
        ].join("\r\n");
        
        format!("{}\r\n\r\n", response.to_string() + &headers)
            .into_bytes()
    }
}

/// 解析 URL 字符串 (用於演示模式)
pub fn parse_url(url: &str) -> (String, String) {
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    
    if let Some(slash_pos) = url.find('/') {
        let host = url[..slash_pos].to_string();
        let path = url[slash_pos..].to_string();
        (host, path)
    } else {
        (url.to_string(), "/".to_string())
    }
}

/// 在字節數組中查找序列
fn find_sequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// 驗證 HTTP method
fn is_valid_method(method: &str) -> bool {
    matches!(
        method.to_uppercase().as_str(),
        "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH"
    )
}

impl Default for HttpParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_request() {
        let parser = HttpParser::new();
        let request = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
        
        let req = parser.parse_request(request).unwrap();
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/index.html");
        assert_eq!(req.host, "example.com");
    }

    #[test]
    fn test_parse_request_with_ads() {
        let parser = HttpParser::new();
        let request = b"GET /ads/banner.jpg HTTP/1.1\r\nHost: adserver.com\r\n\r\n";
        
        let req = parser.parse_request(request).unwrap();
        assert_eq!(req.path, "/ads/banner.jpg");
        assert_eq!(req.host, "adserver.com");
    }

    #[test]
    fn test_parse_url() {
        let (host, path) = parse_url("https://example.com/path/to/page");
        assert_eq!(host, "example.com");
        assert_eq!(path, "/path/to/page");
    }
}
