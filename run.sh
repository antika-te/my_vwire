#!/bin/bash
set -e

echo "🚀 運行 Vwire 廣告過濾系統 (演示模式)"
echo ""

cd "$(dirname "$0")"

# 檢查編譯產物
if [ ! -f "./target/release/vwire-filter" ]; then
    echo "❌ 錯誤：未找到編譯產物，請先執行編譯"
    echo "   cargo build --release"
    exit 1
fi

# 運行程序
echo "✅ 啟動程序..."
./target/release/vwire-filter --debug
