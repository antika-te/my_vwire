# Vwire 廣告過濾器 - 安裝指南

本文檔提供 Vwire 廣告過濾系統的詳細安裝和配置說明。

## 📋 目錄

- [系統要求](#系統要求)
- [快速安裝](#快速安裝)
- [從源碼編譯](#從源碼編譯)
- [Docker 部署](#docker-部署)
- [配置說明](#配置說明)
- [CA 證書安裝](#ca-證書安裝)
- [驗證安裝](#驗證安裝)
- [故障排除](#故障排除)

---

## 系統要求

### 硬件要求

- **CPU**: 雙核 2.0 GHz 或更高
- **內存**: 512 MB RAM (推薦 1 GB)
- **存儲**: 100 MB 可用空間
- **網絡**: 支持 XDP 的網卡（Intel i350 或更高）

### 軟件要求

- **操作系統**: Linux kernel 5.8 或更高
- **發行版**: Ubuntu 20.04+, Debian 11+, CentOS 8+
- **Rust**: 1.70 或更高 (編譯時需要)
- **LLVM/Clang**: 12.0 或更高 (eBPF 編譯需要)

### 內核配置

確保內核啟用了以下選項：

```bash
CONFIG_BPF=y
CONFIG_BPF_SYSCALL=y
CONFIG_BPF_JIT=y
CONFIG_XDP_SOCKETS=y
```

檢查方法：

```bash
zgrep CONFIG_BPF /proc/config.gz
zgrep CONFIG_XDP /proc/config.gz
```

---

## 快速安裝

### 方法 1: 下載預編譯二進制文件

```bash
# 下載最新版本
wget https://github.com/antika-te/my_vwire/releases/latest/download/vwire-filter-x86_64-unknown-linux-gnu.tar.gz

# 解壓
tar -xzf vwire-filter-x86_64-unknown-linux-gnu.tar.gz

# 移動到系統路徑
sudo mv vwire-filter /usr/local/bin/

# 驗證安裝
vwire-filter --version
```

### 方法 2: 使用安裝腳本

```bash
# 下載安裝腳本
curl -fsSL https://raw.githubusercontent.com/antika-te/my_vwire/main/install.sh | sudo bash
```

---

## 從源碼編譯

### 1. 安裝依賴

#### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y \
    curl \
    git \
    build-essential \
    llvm \
    clang \
    libbpf-dev \
    linux-headers-$(uname -r) \
    pkg-config \
    libssl-dev
```

#### CentOS/RHEL

```bash
sudo yum install -y \
    curl \
    git \
    gcc \
    make \
    llvm \
    clang \
    kernel-devel-$(uname -r) \
    kernel-headers-$(uname -r) \
    openssl-devel
```

### 2. 安裝 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default 1.75
```

### 3. 克隆倉庫

```bash
git clone https://github.com/antika-te/my_vwire.git
cd my_vwire
```

### 4. 編譯

```bash
# Debug 版本
cargo build

# Release 版本（推薦）
cargo build --release

# 編譯後的二進制文件位於：
# ./target/release/vwire-filter
```

### 5. 安裝

```bash
sudo cp target/release/vwire-filter /usr/local/bin/
sudo chmod +x /usr/local/bin/vwire-filter
```

---

## Docker 部署

### 1. 前置要求

- Docker 20.10+
- Docker Compose 2.0+

### 2. 快速啟動

```bash
# 克隆倉庫
git clone https://github.com/antika-te/my_vwire.git
cd my_vwire

# 啟動容器
docker-compose up -d
```

### 3. 驗證容器狀態

```bash
docker-compose ps
docker logs -f vwire-ad-filter
```

### 4. 自定義配置

編輯 `docker-compose.yml` 修改配置：

```yaml
environment:
  - VWIRE_INTERFACE=eth0  # 網絡接口
  - VWIRE_MODE=production # 運行模式
  - RUST_LOG=info         # 日誌級別
```

### 5. 停止和清理

```bash
# 停止容器
docker-compose down

# 停止並刪除數據卷
docker-compose down -v
```

---

## 配置說明

### 配置文件位置

```
/etc/vwire/config.toml
```

### 配置示例

```toml
# Vwire 廣告過濾器配置

# 網絡接口
[network]
interface = "eth0"
xdp_mode = "native"  # native, skb, offload

# 廣告規則
[ads]
enable_filtering = true
rules_file = "/etc/vwire/ad-rules.txt"
whitelist_file = "/etc/vwire/whitelist.txt"

# 後端代理
[backend]
enable = true
address = "127.0.0.1:8080"
timeout = 30  # seconds
connection_pool_size = 10

# HTTPS 解密
[https]
enable = true
ca_cert_path = "/etc/vwire/vwire-ca.crt"
ca_key_path = "/etc/vwire/vwire-ca.key"

# 日誌
[logging]
level = "info"  # trace, debug, info, warn, error
file = "/var/log/vwire/vwire.log"
max_size = "10MB"
max_files = 5
```

### 命令行參數

```bash
vwire-filter --help

選項:
  --interface <INTERFACE>      網絡接口 (default: eth0)
  --config <CONFIG>            配置文件路徑
  --demo                       演示模式（不需要 eBPF）
  --https                      啟用 HTTPS 過濾
  --export-ca-cert <PATH>      導出 CA 證書
  --backend <ADDRESS>          後端服務地址
  --log-level <LEVEL>          日誌級別 (default: info)
  --help                       顯示幫助信息
  --version                    顯示版本號
```

---

## CA 證書安裝

### 為什麼需要安裝 CA 證書？

Vwire 使用 HTTPS MITM（中間人）技術來解密和過濾加密的廣告流量。這需要您的系統信任 Vwire 生成的 CA 證書。

### Linux (Ubuntu/Debian)

```bash
# 導出證書
vwire-filter --export-ca-cert

# 安裝到系統信任存儲
sudo cp vwire-ca.crt /usr/local/share/ca-certificates/
sudo update-ca-certificates

# 驗證
ls -l /etc/ssl/certs/ | grep vwire
```

### Linux (CentOS/RHEL)

```bash
sudo cp vwire-ca.crt /etc/pki/ca-trust/source/anchors/
sudo update-ca-trust extract
```

### macOS

```bash
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain vwire-ca.crt
```

### Windows

1. 雙擊 `vwire-ca.crt`
2. 點擊「安裝證書」
3. 選擇「本地計算機」
4. 選擇「將所有證書放入以下存儲」
5. 點擊「瀏覽」
6. 選擇「受信任的根證書頒發機構」
7. 點擊「確定」完成安裝

### Android

1. 將 `vwire-ca.crt` 傳輸到設備
2. 設置 → 安全 → 加密與憑據
3. 點擊「從存儲安裝」
4. 選擇證書文件
5. 設置證書名稱
6. 確認安裝

### iOS

1. 通過 AirDrop 或郵件發送證書文件
2. 設置 → 已下載的描述文件
3. 安裝描述文件
4. 一般 → 關於本機 → 證書信任設置
5. 啟用 Vwire CA 證書的完全信任

---

## 驗證安裝

### 1. 檢查版本

```bash
vwire-filter --version
# 輸出：vwire-filter 0.4.0
```

### 2. 檢查幫助

```bash
vwire-filter --help
```

### 3. 測試演示模式

```bash
vwire-filter --demo
```

應該看到類似輸出：

```
[INFO] 🚀 啟動 Vwire 廣告過濾系統 v0.4.0
[INFO]   運行模式：演示模式
[INFO]   HTTPS 過濾：已啟用
[INFO] ✅ 系統就緒
```

### 4. 測試網絡流量

```bash
# 啟動過濾器（需要 root）
sudo vwire-filter --interface eth0

# 另一個終端測試
curl -I https://www.google.com
curl -I https://googleadservices.com
```

### 5. 檢查日誌

```bash
journalctl -u vwire-filter -f
# 或查看日誌文件
tail -f /var/log/vwire/vwire.log
```

---

## 故障排除

### 問題 1: eBPF 程序加載失敗

**錯誤**: `Error: Failed to load eBPF program`

**解決方案**:

```bash
# 檢查內核版本
uname -r  # 需要 >= 5.8

# 檢查 BPF 文件系統
mount | grep bpf

# 如果未掛載
sudo mount -t bpf bpf /sys/fs/bpf
```

### 問題 2: XDP 不支持

**錯誤**: `Error: XDP not supported on this interface`

**解決方案**:

```bash
# 檢查網卡驅動
ethtool -i eth0 | grep driver

# 某些網卡需要特定驅動
# Intel: 使用 ixgbe 或 i40e 驅動
# Mellanox: 使用 mlx5_core 驅動

# 嘗試使用 SKB 模式
vwire-filter --interface eth0 --xdp-mode skb
```

### 問題 3: 證書驗證失敗

**錯誤**: `SSL: certificate verification failed`

**解決方案**:

1. 重新安裝 CA 證書（見上方 CA 證書安裝章節）
2. 清除瀏覽器 SSL 緩存
3. 重啟瀏覽器或系統

### 問題 4: 性能低下

**症狀**: 網絡延遲增加，吞吐量下降

**解決方案**:

```bash
# 檢查 CPU 使用率
top -p $(pgrep vwire-filter)

# 調整 XDP 模式
vwire-filter --interface eth0 --xdp-mode native

# 如果仍然性能低下，檢查：
# - 網卡是否支持 XDP offload
# - 是否與其他 XDP 程序衝突
# - 系統資源是否充足
```

### 問題 5: Docker 容器無法啟動

**錯誤**: `ERROR: failed to start containers`

**解決方案**:

```bash
# 檢查權限
sudo usermod -aG docker $USER
newgrp docker

# 檢查端口衝突
docker-compose ps
netstat -tlnp | grep :8080

# 重新構建
docker-compose build
docker-compose up -d
```

---

## 獲取幫助

### 文檔

- [README.md](README.md) - 項目介紹
- [RELEASE_NOTES.md](RELEASE_NOTES.md) - 發布說明
- [STATUS.md](STATUS.md) - 開發狀態

### 社區支持

- GitHub Issues: https://github.com/antika-te/my_vwire/issues
- 討論區：https://github.com/antika-te/my_vwire/discussions

### 商業支持

如需企業級支持和定製服務，請聯繫：
- Email: support@vwire.io
- Website: https://vwire.io

---

**最後更新**: 2026-06-11  
**文檔版本**: v0.4.0  
**適用版本**: Vwire v0.4.0+
