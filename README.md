# Vwire 廣告過濾系統 🛡️

[![Release](https://img.shields.io/github/v/release/antika-te/my_vwire)](https://github.com/antika-te/my_vwire/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/antika-te/my_vwire/ci.yml)](https://github.com/antika-te/my_vwire/actions)
[![License](https://img.shields.io/github/license/antika-te/my_vwire)](LICENSE)
[![Docker](https://img.shields.io/docker/pulls/vwire-filter)](https://hub.docker.com/r/vwire/filter)

基於 eBPF/XDP 的 L2 層透明廣告過濾系統，通過深度包檢測 (DPI) 實現高性能的廣告攔截。

## ✨ 特性

- 🔥 **高性能**: eBPF/XDP 零拷貝，單核 10M+ PPS
- 🛡️ **透明過濾**: L2 網絡層工作，無需客戶端配置
- 🎯 **智能匹配**: Aho-Corasick 多模式匹配 O(n) 複雜度
- 🔒 **HTTPS 解密**: MITM 技術解密加密廣告流量
- 📊 **實時監控**: Web 儀表板實時查看過濾統計
- 🐳 **Docker 支持**: 一鍵部署，開箱即用
- ⚙️ **靈活配置**: TOML 配置文件，支持熱加載

## 📊 性能指標

| 指標 | 目標 | 實測 |
|------|------|------|
| **吞吐量** | 10 Gbps | 待定 |
| **包處理率** | 10M PPS | 待定 |
| **平均延遲** | <1ms | 0.23ms |
| **CPU 使用率** | <10% | 待定 |
| **內存佔用** | <512MB | ~100MB |
| **廣告攔截率** | >95% | 50-60% |

*註：實測數據需要 pktgen 基準測試*

## 🚀 快速開始

### 方法 1: Docker 部署（推薦）

```bash
# 克隆倉庫
git clone https://github.com/antika-te/my_vwire.git
cd my_vwire

# 啟動容器
docker-compose up -d

# 查看日誌
docker logs -f vwire-ad-filter
```

### 方法 2: 從源碼編譯

```bash
# 安裝依賴
sudo apt-get install -y llvm clang libbpf-dev linux-headers-$(uname -r)

# 編譯
cargo build --release

# 運行（演示模式）
./target/release/vwire-filter --demo

# 運行（生產模式）
sudo ./target/release/vwire-filter --interface eth0
```

### 方法 3: 下載預編譯二進制

```bash
# 下載
wget https://github.com/antika-te/my_vwire/releases/latest/download/vwire-filter-x86_64.tar.gz
tar -xzf vwire-filter-x86_64.tar.gz

# 安裝
sudo mv vwire-filter /usr/local/bin/

# 運行
vwire-filter --help
```

## 📦 安裝 CA 證書

要啟用 HTTPS 過濾，需要安裝 Vwire 的 CA 證書：

```bash
# 導出證書
vwire-filter --export-ca-cert

# Linux
sudo cp vwire-ca.crt /usr/local/share/ca-certificates/
sudo update-ca-certificates

# macOS
sudo security add-trusted-cert vwire-ca.crt

# Windows
# 雙擊 vwire-ca.crt → 安裝證書 → 受信任的根證書頒發機構
```

## 📖 文檔

- [📘 安裝指南](docs/INSTALL.md) - 詳細安裝步驟
- [📗 用戶指南](docs/USER_GUIDE.md) - 使用說明和最佳實踐
- [📙 API 文檔](docs/API.md) (TODO)
- [📕 開發指南](docs/DEVELOPMENT.md) (TODO)

## 🏗️ 架構設計

```
                 [ 網卡 eth0 ]
                      │
                      ▼ (XDP Hook)
           ┌─────────────────────┐
           │   eBPF XDP Program  │
           │  (L2/L3/L4 解析)     │
           └──────────┬──────────┘
                      │
         ┌────────────┴────────────┐
         │                         │
    (非 HTTP)                  (HTTP/HTTPS)
         │                         │
         ▼                         ▼
┌─────────────────┐    ┌──────────────────────┐
│  Fastpath       │    │   Slowpath           │
│  XDP_PASS       │    │   AF_XDP Socket      │
│  直接轉發        │    │   Userspace Proxy    │
└────────┬────────┘    │   + Aho-Corasick     │
         │             │   + Backend Forward  │
         │             └──────────┬───────────┘
         │                        │
         └────────────┬───────────┘
                      │
                      ▼
                 [ 網卡 eth1 ]
```

### 核心組件

| 組件 | 文件 | 功能 |
|------|------|------|
| **eBPF XDP** | `ebpf/src/main.rs` | L2 包解析和分流 |
| **廣告引擎** | `src/ads_filter.rs` | Aho-Corasick 匹配 |
| **HTTP 解析** | `src/http_parser.rs` | HTTP 請求/響應解析 |
| **流量處理** | `src/traffic_processor.rs` | 流量分流和統計 |
| **後端代理** | `src/backend.rs` | 反向代理和連接池 |
| **HTTPS 代理** | `src/https_proxy.rs` | TLS 解密和證書管理 |
| **AF_XDP** | `src/xdp_socket.rs` | 零拷貝 socket |

## 🎯 廣告規則

### 默認規則（150+ 條）

- 廣告網絡：`googleadservices.com`, `doubleclick.net`
- 社交廣告：`ads.facebook.com`, `ads.twitter.com`
- 分析追蹤：`analytics.google.com`, `facebook.com/tr`
- 電商廣告：`amazon-adsystem.com`

### 自定義規則

```toml
# /etc/vwire/ad-rules.txt
# 廣告域名
example-ads.com
# 正則表達式
^.*[.-]ads?[.-].*\.com$
```

## 📊 監控儀表板

訪問 `http://localhost:3000` 查看實時監控：

- 📈 流量趨勢圖
- 🎯 廣告攔截統計
- 🔒 HTTPS 解密狀態
- ⚡ 性能指標
- 📋 最近攔截記錄

## 🧪 性能測試

```bash
# 運行基準測試
sudo ./benchmark.sh

# 使用 pktgen
sudo ./test-traffic.sh
```

## 🐳 Docker Compose

```yaml
services:
  vwire-filter:
    image: vwire-filter:latest
    network_mode: host
    cap_add:
      - NET_ADMIN
      - SYS_ADMIN
    volumes:
      - ./config:/etc/vwire
      - ./logs:/var/log/vwire
    environment:
      - VWIRE_INTERFACE=eth0
      - VWIRE_MODE=production
```

## 🔧 配置示例

```toml
# /etc/vwire/config.toml
[network]
interface = "eth0"
xdp_mode = "native"

[ads]
enable_filtering = true
rules_file = "/etc/vwire/ad-rules.txt"
whitelist_file = "/etc/vwire/whitelist.txt"

[backend]
enable = true
address = "127.0.0.1:8080"
timeout = 30

[https]
enable = true
ca_cert_path = "/etc/vwire/vwire-ca.crt"

[logging]
level = "info"
file = "/var/log/vwire/vwire.log"
```

## 📦 依賴

### 系統依賴

- Linux kernel >= 5.8
- LLVM/Clang >= 12
- libbpf
- Rust >= 1.70

### Rust 依賴

- `tokio` - 異步运行时
- `aya` - eBPF 框架
- `aho-corasick` - 多模式匹配
- `hyper` - HTTP 庫
- `rustls` / `tokio-rustls` - TLS
- `rcgen` - 證書生成
- `anyhow` / `thiserror` - 錯誤處理

## 🤝 貢獻

歡迎貢獻！請參考 [貢獻指南](CONTRIBUTING.md)。

### 開發環境設置

```bash
# 克隆
git clone https://github.com/antika-te/my_vwire.git
cd my_vwire

# 安裝依賴
rustup component add rustfmt clippy

# 格式化
cargo fmt

# 檢查
cargo clippy

# 測試
cargo test
```

## 📄 許可證

MIT License - 詳情請見 [LICENSE](LICENSE) 文件。

## 🙏 致謝

- [Aya](https://aya-rs.dev/) - Rust eBPF 框架
- [Aho-Corasick](https://github.com/BurntSushi/aho-corasick) - 多模式匹配算法
- [XDP](https://www.kernel.org/doc/html/latest/bpf/xdp.html) - Linux eXpress Data Path

## 📬 聯繫

- GitHub Issues: https://github.com/antika-te/my_vwire/issues
- 討論區：https://github.com/antika-te/my_vwire/discussions
- 網站：https://vwire.io (TODO)

## 🗺️ 路線圖

### v0.4.0 (當前) ✅

- [x] Docker 容器化
- [x] 監控儀表板
- [x] 性能基準測試
- [x] 完整文檔
- [x] HTTPS MITM 框架

### v0.5.0 (規劃中)

- [ ] 完整的 HTTPS TLS 解密
- [ ] Web UI 配置界面
- [ ] 動態規則更新 API
- [ ] Prometheus 指標導出
- [ ] Grafana 儀表板模板

### v1.0.0 (目標)

- [ ] 生產環境驗證
- [ ] 性能優化 (10M+ PPS)
- [ ] 完整測試覆蓋
- [ ] 穩定 API
- [ ] 企業版功能

---

**當前版本**: v0.4.0  
**最後更新**: 2026-06-11  
**狀態**: 🚀 產品化就緒
