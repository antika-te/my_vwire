# Vwire 廣告過濾器 - Dockerfile
# 基於 Rust + eBPF/XDP 的高性能廣告過濾系統

FROM rust:1.75-slim as builder

# 安裝 eBPF 依賴
RUN apt-get update && apt-get install -y \
    llvm \
    clang \
    libbpf-dev \
    linux-headers-amd64 \
    git \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 設置工作目錄
WORKDIR /app

# 複製 Cargo 配置
COPY Cargo.toml Cargo.lock ./

# 複製源代碼
COPY src/ ./src/
COPY ebpf/ ./ebpf/

# 編譯 Release 版本
RUN cargo build --release

# 運行時鏡像
FROM debian:bookworm-slim

# 安裝運行時依賴
RUN apt-get update && apt-get install -y \
    ca-certificates \
    iproute2 \
    ethtool \
    && rm -rf /var/lib/apt/lists/*

# 從編譯器複製二進制文件
COPY --from=builder /app/target/release/vwire-filter /usr/local/bin/
COPY --from=builder /app/vwire-ca.crt /etc/vwire/

# 設置權限
RUN chmod +x /usr/local/bin/vwire-filter

# 創建配置目錄
RUN mkdir -p /etc/vwire
RUN mkdir -p /var/log/vwire

# 暴露端口（如果需要）
EXPOSE 8080

# 健康檢查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep vwire-filter || exit 1

# 運行程序
ENTRYPOINT ["vwire-filter"]
CMD ["--interface", "eth0"]
