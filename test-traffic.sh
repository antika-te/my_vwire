#!/bin/bash
# 流量處理器測試腳本

set -e

cd "$(dirname "$0")"

echo "🧪 測試 Vwire 流量處理器"
echo ""

# 運行流量處理器測試 (不需要 eBPF)
./target/release/vwire-filter --demo << EOF
https://doubleclick.net/ads/banner.html
https://googleadservices.com/tracking.js
https://github.com/antika-te/my_vwire
https://youtube.com/watch?v=test
https://adserver.com/get_ads?size=300x250
q
EOF

echo ""
echo "✅ 測試完成！"
