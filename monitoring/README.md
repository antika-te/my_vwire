# Vwire 監控配置

## Prometheus + Grafana 監控棧

### Prometheus 配置

#### prometheus.yml

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'vwire-filter'
    static_configs:
      - targets: ['localhost:3001']
    metrics_path: '/metrics'
    scrape_interval: 5s
```

#### docker-compose.monitoring.yml

```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    container_name: vwire-prometheus
    restart: unless-stopped
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.enable-lifecycle'
    ports:
      - "9090:9090"
    networks:
      - vwire-net

  grafana:
    image: grafana/grafana:latest
    container_name: vwire-grafana
    restart: unless-stopped
    environment:
      - GF_SECURITY_ADMIN_USER=admin
      - GF_SECURITY_ADMIN_PASSWORD=vwire123
      - GF_INSTALL_PLUGINS=grafana-piechart-panel
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
      - ./grafana/dashboards:/var/lib/grafana/dashboards
    ports:
      - "3000:3000"
    depends_on:
      - prometheus
    networks:
      - vwire-net

  vwire-filter:
    build: .
    container_name: vwire-ad-filter
    restart: unless-stopped
    network_mode: host
    cap_add:
      - NET_ADMIN
      - SYS_ADMIN
    environment:
      - VWIRE_ENABLE_METRICS=true
      - VWIRE_METRICS_PORT=3001
    volumes:
      - ./config:/etc/vwire
      - ./logs:/var/log/vwire

volumes:
  prometheus-data:
    driver: local
  grafana-data:
    driver: local

networks:
  vwire-net:
    driver: bridge
```

---

## Grafana 儀表板

### 進口儀表板

JSON 文件位置：`monitoring/grafana/dashboards/vwire-overview.json`

進口步驟：
1. 登錄 Grafana (http://localhost:3000)
2. 點擊 Dashboard → Import
3. 上傳 JSON 文件或粘贴 ID
4. 選擇 Prometheus 數據源
5. 點擊 Import

### 儀表板面板說明

#### 1. 核心指標 (Row 1)
- **總請求數**: `sum(rate(vwire_total_requests_total[5m]))`
- **廣告攔截數**: `sum(rate(vwire_blocked_ads_total[5m]))`
- **攔截率**: `sum(vwire_blocked_ads_total) / sum(vwire_total_requests_total) * 100`
- **允許請求數**: `sum(rate(vwire_allowed_requests_total[5m]))`

#### 2. 性能指標 (Row 2)
- **平均延遲**: `histogram_quantile(0.50, sum(rate(vwire_request_latency_seconds_bucket[5m])) by (le))`
- **P95 延遲**: `histogram_quantile(0.95, sum(rate(vwire_request_latency_seconds_bucket[5m])) by (le))`
- **P99 延遲**: `histogram_quantile(0.99, sum(rate(vwire_request_latency_seconds_bucket[5m])) by (le))`

#### 3. HTTPS 解密 (Row 3)
- **HTTPS 解密數**: `sum(rate(vwire_https_decrypted_total[5m]))`
- **解密成功率**: 自定義指標

#### 4. 帶寬節省 (Row 4)
- **節省帶寬**: `sum(rate(vwire_bandwidth_saved_bytes[5m]))`
- **累計節省**: `sum(vwire_bandwidth_saved_bytes)`

#### 5. 廣告域名 Top 10 (Row 5)
- **Top 10 廣告域名**: `topk(10, sum(rate(vwire_blocked_ads_total[5m])) by (domain))`

#### 6. 時間趨勢圖 (Row 6)
- **請求趨勢 (24h)**: `sum(rate(vwire_total_requests_total[5m]))`
- **攔截趨勢 (24h)**: `sum(rate(vwire_blocked_ads_total[5m]))`

---

## 警報規則

### alerts.yml

```yaml
groups:
  - name: vwire_alerts
    rules:
      # 高延遲警報
      - alert: VwireHighLatency
        expr: histogram_quantile(0.95, sum(rate(vwire_request_latency_seconds_bucket[5m])) by (le)) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Vwire 高延遲警報"
          description: "P95 延遲超過 100ms (當前值：{{ $value }}s)"

      # 高錯誤率警報
      - alert: VwireHighErrorRate
        expr: sum(rate(vwire_blocked_ads_total[5m])) / sum(rate(vwire_total_requests_total[5m])) < 0.1
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Vwire 攔截率過低"
          description: "攔截率低於 10% (當前值：{{ $value | humanizePercentage }})"

      # 服務宕機警報
      - alert: VwireServiceDown
        expr: up{job="vwire-filter"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Vwire 服務宕機"
          description: "Vwire 過濾器已宕機超過 1 分鐘"

      # 內存使用過高
      - alert: VwireHighMemory
        expr: process_resident_memory_bytes{job="vwire-filter"} > 536870912
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Vwire 內存使用過高"
          description: "內存使用超過 512MB (當前值：{{ $value | humanize1024 }})"
```

---

## 使用方法

### 1. 啟動監控棧

```bash
cd monitoring
docker-compose -f docker-compose.monitoring.yml up -d
```

### 2. 訪問服務

- **Grafana**: http://localhost:3000 (admin/vwire123)
- **Prometheus**: http://localhost:9090
- **Vwire Metrics**: http://localhost:3001/metrics

### 3. 進口儀表板

```bash
# Grafana UI
Dashboard → Import → Upload JSON file
選擇 monitoring/grafana/dashboards/vwire-overview.json
```

### 4. 配置警報

警報規則已自動通過 Prometheus 配置文件加載。

配置通知渠道：
1. Grafana → Alerting → Notification channels
2. 添加 Email/Slack/Webhook 通知
3. 在警報規則中引用通知渠道

---

## 指標說明

### Counter 類型

| 指標名稱 | 說明 | 單位 |
|---------|------|------|
| `vwire_total_requests_total` | 總請求數 | requests |
| `vwire_blocked_ads_total` | 攔截廣告數 | requests |
| `vwire_allowed_requests_total` | 允許請求數 | requests |
| `vwire_https_decrypted_total` | HTTPS 解密數 | requests |
| `vwire_bandwidth_saved_bytes` | 節省帶寬 | bytes |

### Histogram 類型

| 指標名稱 | 說明 | 單位 |
|---------|------|------|
| `vwire_request_latency_seconds` | 請求延遲分佈 | seconds |

### Gauge 類型

| 指標名稱 | 說明 | 單位 |
|---------|------|------|
| `process_resident_memory_bytes` | 內存使用 | bytes |
| `process_cpu_seconds_total` | CPU 使用 | seconds |

---

## 故障排除

### Promethens 無法抓取指標

```bash
# 檢查 Vwire API 服務
curl http://localhost:3001/metrics

# 檢查 Prometheus 配置
docker logs vwire-prometheus

# 重啟 Prometheus
docker restart vwire-prometheus
```

### Grafana 無法連接 Prometheus

1. 檢查數據源配置 (Configuration → Data sources)
2. Prometheus URL 應為：`http://prometheus:9090`
3. 點擊 "Save & Test"

### 儀表板無數據

1. 確認時間範圍正確（選擇最近 1 小時）
2. 確認 Prometheus 數據源選擇正確
3. 檢查指標名稱是否匹配

---

**文檔版本**: v0.4.0  
**最後更新**: 2026-06-12
