# Vwire - eBPF/XDP 虛擬導線廣告過濾系統

## ✅ 已完成 (PoC 版本)

### Userspace 廣告過濾引擎

- [x] **Aho-Corasick 多模式匹配引擎**
  - 集成 `aho-corasick` crate
  - 支援高效域名和 URL 路徑匹配
  - O(n) 時間複雜度，適合大规模規則庫

- [x] **規則管理系統**
  - 黑名單：12 條預設廣告規則
  - 白名單：5 條常見網站放行規則
  - 易於擴展的規則添加接口

- [x] **互動演示模式**
  - 命令行互動界面
  - 實時統計信息反饋
  - 攔截率計算

- [x] **統計監控**
  - 總請求數追蹤
  - 攔截/放行計數
  - 攔截率 percentage 計算

### 測試結果

```bash
$ echo -e "https://ads.example.com/get_ads\nhttps://google.com/search\nhttps://doubleclick.net/banner\nq" | ./vwire-filter

[INFO] 🚀 啟動 Vwire 廣告過濾系統
[INFO] 🔧 初始化廣告過濾引擎...
[INFO]   ✓ 已加載 12 條廣告規則
[INFO]   ✓ 已加載 5 條白名單規則

URL> ❌ 廣告 - 已攔截          (ads.example.com/get_ads)
URL> ✓ 正常內容 - 放行          (google.com/search)
URL> ❌ 廣告 - 已攔截          (doubleclick.net/banner)

📊 過濾統計:
  總請求數：3
  攔截廣告：2
  放行請求：1
  攔截率：66.67%
```

## 🔲 待完成 (下一步開發)

### eBPF XDP 程序

- [ ] **XDP 流量分類**
  - 以太網/ IP / TCP header 解析
  - Port 80 (HTTP) 識別
  - Port 443 (HTTPS) 識別 (階段 3)

- [ ] **AF_XDP 零拷貝通道**
  - XDP_UMEM 配置
  - RX/TX ring buffer 設置
  - Kernel ↔ Userspace 高效傳輸

- [ ] **流量重定向**
  - XDP_REDIRECT 動作
  - BPF_MAP_TYPE_DEVMAP 配置
  - 非 HTTP 流量 bypass

### HTTPS MITM (階段 3)

- [ ] **TLS 代理引擎**
  - tokio-rustls 集成
  - 動態證書生成 (rcgen)
  - ClientHello SNI 提取

- [ ] **內容過濾**
  - HTML 響應體修改
  - CSS 注入
  - JavaScript 廣告替換

### 性能優化

- [ ] **白名單 bypass map**
  - eBPF HASH map 實現
  - Userspace 動態更新
  - 亞微秒級匹配

- [ ] **Watchdog 監控**
  - Heart beat 機制
  - 崩潰自動恢復
  - 透傳模式切換

## 快速開始

### 編譯

```bash
# 安裝 Rust (如果未安裝)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 編譯
cd vwire-filter
cargo build --release
```

### 運行演示

```bash
# 互動模式
./target/release/vwire-filter

# 調試模式 (詳細日誌)
./target/release/vwire-filter --debug
```

### 測試規則

```bash
# 批量測試
echo -e "https://doubleclick.net/ads\nhttps://youtube.com/watch\nhttps://adserver.com/tracker" | ./target/release/vwire-filter
```

## 技術棧

| 組件 | 技術 | 說明 |
|------|------|------|
| **Userspace** | Rust + Tokio | 異步網絡編程 |
| **eBPF** | Aya framework | Rust eBPF 框架 |
| **匹配算法** | Aho-Corasick | 多模式匹配 |
| **TLS** | tokio-rustls | (階段 3) |
| **證書生成** | rcgen | (階段 3) |

## 架構圖

```
┌─────────────────────────────────────────┐
│          網路接口 (eth0)                 │
└─────────────────┬───────────────────────┘
                  │
                  ▼
         ┌─────────────────┐
         │   XDP Program   │ ← eBPF (kernel space)
         │  (Port 識別/分流)│
         └───────┬─────────┘
                 │
        ┌────────┴────────┐
        │                 │
        ▼                 ▼
  ┌──────────┐    ┌──────────────┐
  │ Fastpath │    │  Slowpath    │
  │ 非 HTTP   │    │  HTTP (80)   │
  │ 直接放行  │    │  AF_XDP      │
  └──────────┘    └───────┬───────┘
                          │
                          ▼
                 ┌─────────────────┐
                 │  Userspace      │
                 │  Aho-Corasick   │
                 │  規則匹配        │
                 └─────────────────┘
```

## 規則格式

### 黑名單規則類型

1. **域名匹配**: `doubleclick.net`, `googleadservices.com`
2. **路徑匹配**: `/ads/`, `/get_ads`, `/banner?`
3. **關鍵字匹配**: `adservice`, `adtracker`, `analytics`

### 白名單規則

- `google.com` (搜索)
- `youtube.com` (影片)
- `github.com` (開發)
- `netflix.com` (串流)

## 性能指標 (目標)

| 指標 | 階段 1 (HTTP) | 階段 2 (優化) | 階段 3 (HTTPS) |
|------|-------------|-------------|---------------|
| **吞吐量** | 1 Gbps | 10 Gbps | 2 Gbps |
| **延遲** | < 1ms | < 100μs | < 5ms |
| **CPU 使用率** | < 20% | < 10% | < 50% |
| **內存占用** | < 100MB | < 50MB | < 500MB |

## 下一步行動

1. **編譯 eBPF 程序**
   - 安裝 LLVM 和 bpf 工具鏈
   - 編譯 `ebpf/src/main.rs`
   - 測試 XDP 程序加載

2. **實現 AF_XDP**
   - 建立 socket
   - 實現零拷貝接收循環
   - HTTP 請求解析

3. **HTTP 響應注入**
   - HTTP 204 攔截響應
   - 頭部構造
   - XDP_TX 發送

## 參考資源

- [Aya Book](https://aya-rs.dev/book/) - Rust eBPF 權威指南
- [XDP Project](https://www.xdp-project.org/) - XDP 技術資源
- [EasyList](https://easylist.to/) - 廣告過濾規則訂閱
- [Aho-Corasick Algorithm](https://en.wikipedia.org/wiki/Aho%E2%80%93Corasick_algorithm) - 多模式匹配算法

## 商用潛力

### 優勢

1. **L2 層過濾**: 比 DNS 攔截更難繞過
2. **HTTPS 解密**: 適配現代加密流量
3. **零拷貝架構**: 亞微秒級延遲
4. **Rust 安全性**: 無內存安全問題

### 應用場景

- 企業廣告過濾網關
- ISP 級別內容過濾
- 學校/圖書館網絡管理
- 家庭路由器固件

---

**狀態**: PoC 原型完成 ✓
**下一里程碑**: eBPF XDP 程序編譯與 AF_XDP 集成
**預計時間**: 1-2 週

✊ Let's code!
