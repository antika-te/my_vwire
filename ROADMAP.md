# Vwire 開發路線圖

## 當前狀態 (PoC 階段)

✅ **已完成**:
- [x] 專案架構搭建 (Rust + Aya eBPF 框架)
- [x] Userspace 廣告過濾引擎原型
- [x] Aho-Corasick 多模式匹配算法集成
- [x] 基礎規則庫 (域名黑名單 + URL 特徵)
- [x] 演示模式 (互動式測試)

🔧 **待完成 (eBPF 部分)**:
- [ ] XDP 程序編譯 (需要 bpfel-unknown-none target)
- [ ] AF_XDP Socket 設立
- [ ] Kernel → Userspace 零拷貝數據通道

## 階段 1: HTTP 明文過濾 (1-2 週)

### 目標
實現完整的 HTTP 流量攔截和廣告過濾

### 任務清單

#### 1.1 XDP 程序開發
- [ ] 完善 `ebpf/src/main.rs`
  - TCP header 解析
  - Port 80 識別
  - BPF_MAP_TYPE_DEVMAP 配置
  - XDP_REDIRECT 實現
  
```rust
// 關鍵代碼片段
#[map]
static mut DEVMAP: BpfMap<DevMapHash, IfIndex> = BpfMap::uninit();

#[xdp]
pub fn vwire_xdp(ctx: XdpContext) -> u32 {
    // 解析以太網 → IP → TCP
    if dest_port == 80 {
        return XDP_REDIRECT; // 重定向到 AF_XDP
    }
    XDP_PASS
}
```

#### 1.2 AF_XDP Userspace 接收
- [ ] 建立 AF_XDP socket
- [ ] 實現零拷貝數據接收循環
- [ ] 解析 HTTP 請求 (Host header)

```rust
use std::os::unix::io::AsRawFd;
use mio::net::UdpSocket; // AF_XDP 本質是特殊的 socket

// 配置 XDP_UMEM 和 XDP_RING
```

#### 1.3 HTTP 響應注入
- [ ] 攔截廣告請求後返回 HTTP 204
- [ ] 構造合法的 HTTP 響應頭
- [ ] 通過 XDP_TX 發回客戶端

### 驗收標準
- [ ] 能識別並隔離所有 HTTP 80 端口流量
- [ ] 命中率測試：EasyList 規則庫匹配率 > 95%
- [ ] 性能基準：單核 1Gbps 吞吐量，延遲 < 1ms

---

## 階段 2: 性能優化 (1-2 週)

### 目標
實現快慢路分流和動態 bypass 機制

### 任務清單

#### 2.1 白名單 Map
- [ ] 創建 eBPF HASH map 存儲 bypass IP
- [ ] Userspace 動態更新機制
- [ ] 優先匹配邏輯

```rust
#[map]
static mut BYPASS_MAP: BpfHash<[u8; 4], u8> = BpfMap::uninit();

// XDP 程序中使用
if BYPASS_MAP.get(&src_ip).is_some() {
    return XDP_PASS; // 直接放行
}
```

#### 2.2 Watchdog 監控
- [ ] 監控 userspace 進程健康狀態
- [ ] 崩潰檢測 (heart beat 機制)
- [ ] 自動切換透傳模式

```rust
async fn run_watchdog() {
    loop {
        if !userspace_alive.load(Ordering::Relaxed) {
            // 切換 eBPF 到全透傳模式
            enable_passthrough_mode();
        }
        sleep(Duration::from_secs(5));
    }
}
```

#### 2.3 性能基準測試
- [ ] 使用 `pktgen` 產生測試流量
- [ ] 測量不同負載下的吞吐量
- [ ] CPU 使用率分析 (perf)

### 驗收標準
- [ ] 白名單 IP 流量延遲 < 100μs
- [ ] Watchdog 反應時間 < 1 秒
- [ ] 10Gbps 流量下不丟包

---

## 階段 3: HTTPS MITM (3-4 週)

### 目標
實現透明 HTTPS 解密和內容過濾

### 任務清單

#### 3.1 TLS Proxy 引擎
- [ ] 基於 `tokio-rustls` 實現 TLS 代理
- [ ]  dinâmica 證書生成 (RCGen)
- [ ] 雙向 TLS 握手 (Client ↔ Proxy ↔ Server)

```rust
use tokio_rustls::TlsAcceptor;
use rcgen::Certificate;

// 為每個域名動態生成證書
fn generate_cert(domain: &str, ca: &Certificate) -> Certificate {
    // 使用 CA 簽發域名證書
}
```

#### 3.2 SNI 提取
- [ ] 在 XDP 層解析 TLS ClientHello
- [ ] 提取 SNI (Server Name Indication)
- [ ] 基於 SNI 的早期攔截

```rust
// eBPF 中解析 TLS extension
fn extract_sni(client_hello: &[u8]) -> Option<String> {
    // 解析 TLS record → Handshake → Extensions
}
```

#### 3.3 HTML 內容注入
- [ ] 響應體解析 (HTML/CSS/JS)
- [ ] 廣告標籤移除 (`<div class="ad-">`)
- [ ] CSS 注入 (`.ad-banner { display: none !important; }`)

```rust
fn inject_css(html: &mut String) -> Result<()> {
    let hiding_css = r#"<style>
        .ad-banner, .advertisement, [class*="ad-"] { 
            display: none !important; 
        }
    </style>"#;
    
    // 注入到 </head> 前
    html.replace("</head>", &format!("{}</head>", hiding_css));
    Ok(())
}
```

#### 3.4 SSL Pinning 繞過
- [ ] 檢測 SSL Pinning (握手失敗時)
- [ ] 建立繞過白名單 (YouTube/TikTok/網銀)
- [ ] 降級到 L4 SNI 阻斷

### 驗收標準
- [ ] 支援 TLS 1.2 / 1.3
- [ ] 99%  HTTPS 網站證書可信
- [ ] SSL Pinning App 能正常使用 (降級放行)

---

## 階段 4: 產品化 (2-3 週)

### 目標
準備生產環境部署

### 任務清單

#### 4.1 規則自動更新
- [ ] 訂閱 EasyList / AdGuard feeds
- [ ] 每日定時更新
- [ ] 熱加載 (不重啟服務)

#### 4.2 監控儀表板
- [ ] Prometheus metrics 導出
- [ ] Grafana 儀表板
- [ ] 實時流量可視化

#### 4.3 Docker 容器化
- [ ] Dockerfile 編寫
- [ ] Docker Compose 配置
- [ ] Kubernetes Helm Chart

#### 4.4 安裝腳本
- [ ] 一鍵安裝腳本
- [ ] 系統服務註冊 (systemd)
- [ ] 開機自啟動

### 驗收標準
- [ ] 安裝部署時間 < 5 分鐘
- [ ] 服務重啟後自動恢復
- [ ] 監控指標覆蓋率 > 90%

---

## 技術債與風險

### 已知風險

1. **eBPF 編譯依賴**
   - 需要 LLVM 和 bpf target
   - 解決方案：提供 Docker 編譯環境

2. **SSL Pinning 普及**
   - 越來越多 App 使用證書鎖定
   - 解決方案：專注於瀏覽器流量，App 流量 bypass

3. **HTTP/3 (QUIC) 興起**
   - 基於 UDP，XDP 解析複雜
   - 解決方案：暫時只處理 TCP 流量

### 性能風險

1. **HTTPS 解密 CPU 開銷**
   - RSA/ECDHE 握手非常消耗 CPU
   - 解決方案：會話復用 + 硬體加速

2. **記憶體內存壓力**
   - 大規則庫 (10 萬+) 占用內存
   - 解決方案：壓縮存儲 + 分流加載

---

## 參考資源

### eBPF/XDP
- [Aya Book](https://aya-rs.dev/book/) - Rust eBPF 權威指南
- [xdp-project](https://www.xdp-project.org/) - XDP 官方資源
- [iovisor.org](https://www.iovisor.org/) - eBPF 技術文檔

### Rust 網絡編程
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Rustls Examples](https://github.com/rustls/rustls)

### 廣告過濾規則
- [EasyList](https://easylist.to/easylist/easylist.txt)
- [AdGuard DNS Filter](https://github.com/AdguardTeam/AdGuardDNS)

---

## 下一步行動

**立即執行 (本週)**:
1. ✅ Userspace PoC 原型完成
2. 🔲 安裝 LLVM 和 bpf 編譯工具鏈
3. 🔲 編譯 eBPF XDP 程序
4. 🔲 測試 AF_XDP socket 通信

**短期目標 (2 週內)**:
- 實現完整的 HTTP 流量攔截
- 達到 1Gbps 吞吐量

**中期目標 (1 個月)**:
- HTTPS MITM 功能上線
- 性能優化到 10Gbps

**長期願景 (3 個月)**:
- 生產環境就緒
- 支持 Kubernetes 部署
- 建立商業化監控平台

✊ 讓我們開始編碼吧！
