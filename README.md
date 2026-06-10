# Vwire Filter - eBPF/XDP 廣告過濾系統

基於 eBPF/XDP 的 L2 層透明廣告過濾系統，通過深度包檢測 (DPI) 實現高性能的廣告攔截。

## 架構設計

```
                 [ 網卡 1 (IN) ]
                       │
                       ▼ (XDP 零拷貝接收)
          ┌─────────────────────────┐
          │     L4 協議與埠過濾      │
          └────────────┬────────────┘
                       │
             ┌─────────┴─────────┐
    (非 HTTP/HTTPS)            (HTTP / HTTPS)
             │                   │
             ▼                   ▼
    ┌─────────────────┐ ┌─────────────────────────┐
    │  快路 (Fastpath) │ │     慢路 (Slowpath)       │
    │  直接 L2 轉發    │ │  AF_XDP → Userspace     │
    └────────┬────────┘ │  廣告規則匹配引擎       │
             │          └────────────┬────────────┘
             └─────────┬─────────────┘
                       │
                       ▼
                 [ 網卡 2 (OUT) ]
```

## 技術棧

- **核心與重定向層**: Linux Kernel + Aya (Rust eBPF 框架)
- **Userspace 代理與匹配層**: Rust (Tokio 異步框架 + Hyper)
- **匹配算法**: Aho-Corasick 自動機

## 專案結構

```
vwire-filter/
├── ebpf/                      # eBPF 程序 (XDP)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs           # XDP 程序入口
├── userspace/                 # Userspace 控制程序
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs           # 主程序入口
│       ├── ads_filter.rs     # 廣告規則匹配引擎
│       └── config.rs         # 配置管理
├── build.rs                   # 編譯腳本
└── README.md
```

## 開發路徑

### 階段 1: HTTP 明文 URL 攔截 (Current PoC)

- [x] XDP Program 原型：解析 TCP header，識別 port 80
- [ ] AF_XDP Socket：建立 kernel → userspace 零拷貝通道
- [x] Userspace Engine：簡單的 URL 黑名單匹配
- [ ] 攔截動作：命中廣告 URL 時返回 HTTP 204

### 階段 2: 性能優化

- [ ] 動態 Bypass Map：eBPF HASH Map 存儲白名單 IP
- [ ] Watchdog 機制：進程崩潰時自動切換透傳模式
- [ ] 性能基準測試：1Gbps/10Gbps 流量下的延遲和吞吐

### 階段 3: HTTPS MITM

- [ ] HTTPS 透明證書劫持 + TLS 代理
- [ ] HTML 內容注入：響應體修改 + CSS 注入
- [ ] SSL Pinning 檢測與繞過策略

## 編譯與運行

### 環境要求

- Linux Kernel 5.8+ (需要完整的 XDP 功能)
- Rust 工具鏈 (包含 `bpf` target)
- libbpf 開發庫

### 編譯 eBPF 程序

```bash
# 安裝 bpf 目標
rustup target add bpf


# 編譯 eBPF 程序
cd ebpf
CARGO_TARGET_DIR=target cargo build --target bpf --release
```

### 編譯 Userspace 程序

```bash
cd userspace
cargo build --release
```

### 運行

```bash
# 需要 root 權限
sudo ./target/release/vwire-filter --interface eth0 --debug
```

## 核心功能模塊

### 模塊 A: XDP 流量分類

位於 `ebpf/src/main.rs`，負責：
- 解析以太網幀、IP/TCP header
- 識別目標端口為 80 的 HTTP 流量
- 輕量級統計信息收集

### 模塊 B: 廣告規則匹配引擎

位於 `userspace/src/ads_filter.rs`，負責：
- 加載 EasyList / AdGuard 規則
- 使用 Aho-Corasick 算法高效匹配
- 動態更新黑名單/白名單

### 模塊 C: Watchdog 監控

位於 `userspace/src/main.rs`，負責：
- 監控主進程健康狀態
- 崩潰時切換 eBPF 到透傳模式
- 確保業務連續性

## 面臨的挑戰

### 1. 性能瓶頸 (CPU 密集型)

**解決方案**: IP/域名白名單動態下發到 eBPF Map，後續流量在 XDP 層直接 bypass。

### 2. HTTPS 普及

**解決方案**: 階段 3 實現透明 MITM，內置自簽 CA 證書動態生成目標網站證書。

### 3. SSL Pinning

**解決方案**: 建立繞過白名單，對無法解密的流量退化到 L4 層 SNI 阻斷或直接放行。

### 4. 業務連續性

**解決方案**: Watchdog 進程監控，崩潰時自動切換全透傳模式。

## 下一步計劃

1. 實現完整的 AF_XDP 零拷貝數據傳輸
2. 添加 Aho-Corasick 自動機實現多模式匹配
3. 集成 EasyList / AdGuard 規則自動更新
4. 實現 HTTP 204 响应注入
5. 添加 Prometheus 指標導出

## 開發者

這是一個極具極客精神和商業價值的項目！✊

## License

MIT
