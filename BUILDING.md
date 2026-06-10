# Vwire 開發環境安裝指南

## 前置條件

### 1. 安裝 Rust 工具鏈

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### 2. 安裝 eBPF 編譯依賴

eBPF 程序需要 LLVM 和特定的 Rust target。

#### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y clang llvm libelf-dev bpfcc-tools
```

#### 添加 BPF 目標 (nightly Rust)

```bash
rustup toolchain install nightly
rustup default nightly
rustup component add rust-src
```

### 3. 檢查內核支援

需要 Linux Kernel 5.8+ 以獲得完整的 XDP 功能：

```bash
uname -r
# 應該 >= 5.8
```

## 編譯說明

### 編譯 eBPF 程序

```bash
cd ebpf

# 方法 1: 使用 cargo bpf (推薦)
cargo install cargo-bpf
cargo bpf build --program vwire_xdp

# 方法 2: 手動編譯 (需要 nightly)
CARGO_TARGET_DIR=target \
  cargo build --target bpfel-unknown-none \
  -Z build-std=core \
  --release
```

### 編譯 Userspace 程序

```bash
cd userspace
cargo build --release
```

### 使用自動化腳本

```bash
./build.sh
```

## 運行要求

- **Root 權限**: XDP 程序需要 root 或 `CAP_BPF` / `CAP_NET_ADMIN` 能力
- **網路接口**: 需要指定一個真實的網路接口 (例如 eth0, ens18)

## 測試環境

### 使用 Docker 測試

如果你無法在本地環境運行，可以使用 Docker:

```bash
docker run --rm -it --privileged \
  -v $(pwd):/vwire-filter \
  -w /vwire-filter \
  quay.io/tweag/bpf /bin/bash

# 在容器內編譯
./build.sh
```

## 常見問題

### Q: `rustup target add bpfel-unknown-none` 失敗

A: 這是正常的，某些 toolchain 不預先提供這個 target。解決方法：

1. 使用 nightly toolchain
2. 或改用手動編譯方式 (見上方)
3. 或使用 `cargo bpf` 工具自動處理

### Q: 編譯時出現 `undefined reference to __sync_fetch_and_add_8`

A: 這是 eBPF 的原子操作問題。解決方法：

```toml
# 在 ebpf/Cargo.toml 中添加
[profile.dev]
opt-level = 3
```

### Q: XDP 程序無法附加到 interface

A: 檢查：
- 是否使用 root 權限運行
- 網路接口名稱是否正確 (`ip link` 查看)
- 內核是否支援 XDP (`ethtool -i <interface>` 查看 driver 是否支援)

## 下一步

编译完成後，參考 [README.md](./README.md) 了解如何運行和測試。
