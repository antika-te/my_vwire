#!/bin/bash
set -e

echo "🔨 編譯 Vwire 廣告過濾系統..."

# 顏色定義
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# 檢查 Rust 是否安裝
if ! command -v cargo &> /dev/null; then
    echo "❌ 錯誤：未找到 cargo，請先安裝 Rust"
    echo "   執行：curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 添加 bpf 目標 (如果未添加)
echo "📦 檢查 bpf 目標..."
rustup target add bpf 2>/dev/null || true

# 編譯 eBPF 程序
echo ""
echo "🔧 編譯 eBPF 程序..."
cd ebpf
CARGO_TARGET_DIR=target cargo build --target bpf-unknown-none --release
cp -f target/bpf-unknown-none/release/vwire-filter-ebpf ../target/ebpf/vwire-filter-ebpf
cd ..
echo -e "${GREEN}✓ eBPF 程序編譯完成${NC}"

# 編譯 userspace 程序
echo ""
echo "🔧 編譯 userspace 程序..."
cd userspace
cargo build --release
cd ..
echo -e "${GREEN}✓ Userspace 程序編譯完成${NC}"

echo ""
echo "🎉 編譯完成！"
echo ""
echo "運行命令:"
echo "  sudo ./target/release/vwire-filter --interface eth0 --debug"
echo ""
