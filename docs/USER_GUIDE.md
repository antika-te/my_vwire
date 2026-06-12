# Vwire 用戶指南

## 快速開始

### 1. 安裝

```bash
# 從源碼編譯
git clone https://github.com/antika-te/my_vwire.git
cd my_vwire
cargo build --release

# 或使用 Docker
docker-compose up -d
```

### 2. 安裝 CA 證書

```bash
# 導出證書
./vwire-filter --export-ca-cert

# 安裝到系統
sudo cp vwire-ca.crt /usr/local/share/ca-certificates/
sudo update-ca-certificates
```

### 3. 啟動過濾器

```bash
# 演示模式（不需要 eBPF）
./vwire-filter --demo

# 生產模式（需要 root 和 eBPF）
sudo ./vwire-filter --interface eth0

# 啟用 HTTPS 過濾
sudo ./vwire-filter --interface eth0 --https
```

### 4. 驗證

```bash
# 測試廣告攔截
curl -I https://googleadservices.com
# 應該返回 204 No Content 或 404

# 測試正常網站
curl -I https://www.google.com
# 應該正常訪問
```

## 使用場景

### 場景 1: 家庭網絡廣告過濾

將 Vwire 部署在家庭網關上：

```bash
# 在路由器或網關上運行
sudo ./vwire-filter --interface br0 --https
```

所有通過該網關的設備都將自動享受廣告過濾。

### 場景 2: 服務器端廣告過濾

在服務器上運行 Vwire 作為反向代理：

```bash
# 配置後端地址
./vwire-filter --backend 127.0.0.1:8080 --https
```

### 場景 3: Docker 容器部署

使用 Docker Compose 快速部署：

```yaml
# docker-compose.yml
version: '3.8'
services:
  vwire:
    image: vwire-filter:latest
    network_mode: host
    cap_add:
      - NET_ADMIN
    volumes:
      - ./config:/etc/vwire
      - ./logs:/var/log/vwire
```

## 配置參考

### 廣告規則

廣告規則文件格式：

```
# 註釋行
googleadservices.com
doubleclick.net
ads.facebook.com
^.*analytics.*  # 正則表達式
```

### 白名單

白名單用於允許特定域名：

```
# 允許的域名
api.example.com
fonts.googleapis.com
```

## 常見問題

### Q: 會影響網絡速度嗎？

A: Vwire 使用 eBPF/XDP 技術在 L2 網絡層進行過濾，性能影響極小（<1%）。

### Q: HTTPS 過濾安全嗎？

A: HTTPS MITM 僅用於本地廣告過濾，CA 證書由您控制。建議僅在受信任的網絡中使用。

### Q: 如何更新廣告規則？

A: 編輯規則文件後重啟 Vwire 即可：
```bash
sudo systemctl restart vwire-filter
```

### Q: 支持哪些瀏覽器？

A: Vwire 在網絡層工作，支持所有瀏覽器和應用程序。

## 監控和日誌

### 查看日誌

```bash
# 實時查看日誌
journalctl -u vwire-filter -f

# 查看錯誤日誌
journalctl -u vwire-filter -p err

# Docker 環境
docker logs -f vwire-ad-filter
```

### 訪問儀表板

打開瀏覽器訪問：http://localhost:3000

## 性能優化

### 調整 XDP 模式

```bash
# SKB 模式（兼容性好）
./vwire-filter --xdp-mode skb

# Native 模式（性能好）
./vwire-filter --xdp-mode native

# Offload 模式（最佳性能，需要硬件支持）
./vwire-filter --xdp-mode offload
```

### 連接池優化

```toml
# config.toml
[backend]
connection_pool_size = 20  # 增加連接池大小
timeout = 60               # 增加超時時間
```

## 故障排除

### 問題：無法加載 eBPF 程序

```bash
# 檢查內核版本
uname -r

# 檢查 BPF 文件系統
mount | grep bpf

# 重新掛載
sudo mount -t bpf bpf /sys/fs/bpf
```

### 問題：證書錯誤

```bash
# 重新安裝證書
vwire-filter --export-ca-cert
sudo update-ca-certificates --fresh

# 清除瀏覽器 SSL 緩存
```

### 問題：性能低下

```bash
# 檢查 CPU 使用率
top -p $(pgrep vwire)

# 檢查網絡接口
ethtool -i eth0

# 嘗試不同 XDP 模式
```

## 最佳實踐

1. **定期更新規則**: 每週更新一次廣告規則
2. **監控系統資源**: 確保有足夠的內存和 CPU
3. **使用 HTTPS**: 啟用 HTTPS 過濾以獲得最佳效果
4. **備份配置**: 定期備份配置文件和 CA 證書
5. **設置監控**: 使用儀表板監控系統狀態

## 升級指南

### 從源碼升級

```bash
git pull origin master
cargo clean
cargo build --release
sudo systemctl restart vwire-filter
```

### Docker 升級

```bash
docker-compose pull
docker-compose up -d
```

## 卸載

### 移除安裝

```bash
# 停止服務
sudo systemctl stop vwire-filter
sudo systemctl disable vwire-filter

# 刪除文件
sudo rm /usr/local/bin/vwire-filter
sudo rm -rf /etc/vwire
sudo rm -rf /var/log/vwire

# 移除 CA 證書
sudo rm /usr/local/share/ca-certificates/vwire-ca.crt
sudo update-ca-certificates --fresh
```

### Docker 卸載

```bash
docker-compose down -v
docker rmi vwire-filter:latest
```

---

**文檔版本**: v0.4.0  
**最後更新**: 2026-06-11  
**適用版本**: Vwire v0.4.0+
