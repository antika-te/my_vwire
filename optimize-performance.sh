#!/bin/bash
# Vwire 性能優化腳本
# 系統級別的性能調優

set -e

echo "🚀 Vwire 性能優化腳本"
echo "===================="
echo ""

# 檢查是否為 root
if [ "$EUID" -ne 0 ]; then
    echo "⚠️  請使用 sudo 運行此腳本"
    exit 1
fi

# 1. CPU 優化
echo "📊 優化 CPU 設置..."

# 設置 IRQ 平衡
if command -v irqbalance &> /dev/null; then
    systemctl enable irqbalance
    systemctl restart irqbalance
    echo "  ✓ IRQ 平衡已啟用"
fi

# 設置 CPU 頻率為性能模式
if [ -f /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor ]; then
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        echo "performance" > $cpu 2>/dev/null || true
    done
    echo "  ✓ CPU 頻率設置為性能模式"
fi

# 2. 內存優化
echo "💾 優化內存設置..."

# 調整 swappiness
echo "10" > /proc/sys/vm/swappiness
echo "  ✓ Swappiness 設置為 10"

# 調整 dirty_ratio
echo "10" > /proc/sys/vm/dirty_ratio
echo "  ✓ Dirty ratio 設置為 10%"

# 3. 網絡優化
echo "🌐 優化網絡設置..."

# 增加 ring buffer 大小
for iface in $(ls /sys/class/net/ | grep -v lo); do
    ethtool -G $iface rx 4096 tx 4096 2>/dev/null || true
    echo "  ✓ $iface Ring Buffer 設置為 4096"
done

# 啟用 Gro/LRO
for iface in $(ls /sys/class/net/ | grep -v lo); do
    ethtool -K $iface gro on lro on 2>/dev/null || true
done
echo "  ✓ GRO/LRO 已啟用"

# 調整 socket 緩衝區
sysctl -w net.core.rmem_max=16777216
sysctl -w net.core.wmem_max=16777216
sysctl -w net.ipv4.tcp_rmem="4096 87380 16777216"
sysctl -w net.ipv4.tcp_wmem="4096 65536 16777216"
echo "  ✓ Socket 緩衝區已優化"

# 增加連接隊列
sysctl -w net.core.somaxconn=65535
sysctl -w net.ipv4.tcp_max_syn_backlog=65535
echo "  ✓ 連接隊列已增加"

# 4. 文件描述符
echo "📁 優化文件描述符..."

ulimit -n 65535 2>/dev/null || true
echo "* soft nofile 65535" >> /etc/security/limits.conf
echo "* hard nofile 65535" >> /etc/security/limits.conf
echo "  ✓ 文件描述符限制設置為 65535"

# 5. eBPF/XDP 優化
echo "⚡ 優化 eBPF/XDP..."

# 增加 BPF JIT 緩衝區
echo "8388608" > /proc/sys/net/core/bpf_jit_limit
echo "  ✓ BPF JIT 緩衝區設置為 8MB"

# 啟用 BPF JIT Harden
echo "2" > /proc/sys/net/core/bpf_jit_harden
echo "  ✓ BPF JIT Harden 已啟用"

# 6. XDP 队列优化
echo "📦 優化 XDP 隊列..."

for iface in $(ls /sys/class/net/ | grep -v lo); do
    # 設置 XDP 隊列數量為 CPU 核心數
    cpu_count=$(nproc)
    ethtool -L $iface combined $cpu_count 2>/dev/null || true
    echo "  ✓ $iface XDP 隊列設置為 $cpu_count"
done

echo ""
echo "✅ 性能優化完成！"
echo ""
echo "建議重啟系統以應用所有優化："
echo "  sudo reboot"
echo ""
echo "查看優化效果："
echo "  ./benchmark.sh"
echo ""
