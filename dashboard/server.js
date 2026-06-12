// Vwire Dashboard Backend API
// 提供統計數據和 Prometheus 指標導出

const express = require('express');
const cors = require('cors');
const promClient = require('prom-client');
const app = express();
const PORT = process.env.PORT || 3001;

// 啟用 CORS
app.use(cors());
app.use(express.json());

// Prometheus 指標
const register = new promClient.Registry();
promClient.collectDefaultMetrics({ register });

// 自定義指標
const totalRequests = new promClient.Counter({
  name: 'vwire_total_requests_total',
  help: 'Total number of requests processed',
  labelNames: ['type']
});

const blockedAds = new promClient.Counter({
  name: 'vwire_blocked_ads_total',
  help: 'Total number of blocked ads',
  labelNames: ['domain', 'type']
});

const allowedRequests = new promClient.Counter({
  name: 'vwire_allowed_requests_total',
  help: 'Total number of allowed requests'
});

const httpsDecrypted = new promClient.Counter({
  name: 'vwire_https_decrypted_total',
  help: 'Total number of HTTPS requests decrypted'
});

const requestLatency = new promClient.Histogram({
  name: 'vwire_request_latency_seconds',
  help: 'Request latency in seconds',
  buckets: [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1]
});

const bandwidthSaved = new promClient.Counter({
  name: 'vwire_bandwidth_saved_bytes',
  help: 'Total bandwidth saved by blocking ads (bytes)'
});

register.registerMetric(totalRequests);
register.registerMetric(blockedAds);
register.registerMetric(allowedRequests);
register.registerMetric(httpsDecrypted);
register.registerMetric(requestLatency);
register.registerMetric(bandwidthSaved);

// 模擬數據存儲
const stats = {
  totalRequests: 0,
  blockedAds: 0,
  allowedRequests: 0,
  httpsDecrypted: 0,
  bandwidthSaved: 0,
  recentBlocks: []
};

// 模擬數據更新（每 5 秒）
setInterval(() => {
  const newRequests = Math.floor(Math.random() * 100);
  const newBlocked = Math.floor(Math.random() * 30);
  const newAllowed = newRequests - newBlocked;
  const newHttps = Math.floor(Math.random() * 20);
  
  stats.totalRequests += newRequests;
  stats.blockedAds += newBlocked;
  stats.allowedRequests += newAllowed;
  stats.httpsDecrypted += newHttps;
  stats.bandwidthSaved += newBlocked * 50 * 1024; // 平均每個廣告 50KB
  
  // 添加最近的攔截記錄
  if (newBlocked > 0) {
    const adDomains = [
      'googleadservices.com',
      'doubleclick.net',
      'ads.facebook.com',
      'analytics.google.com',
      'amazon-adsystem.com',
      'ads.twitter.com',
      'adnxs.com',
      'criteo.com'
    ];
    
    for (let i = 0; i < Math.min(newBlocked, 3); i++) {
      stats.recentBlocks.unshift({
        time: new Date().toISOString(),
        domain: adDomains[Math.floor(Math.random() * adDomains.length)],
        type: ['廣告追蹤', '廣告服務', '社交廣告', 'Analytics', '電商廣告'][Math.floor(Math.random() * 5)],
        action: '已攔截',
        status: 'active'
      });
    }
    
    // 保持最近 100 條記錄
    stats.recentBlocks = stats.recentBlocks.slice(0, 100);
    
    // 更新 Prometheus 指標
    blockedAds.inc({ domain: 'random', type: 'ad' }, newBlocked);
  }
  
  totalRequests.inc({ type: 'http' }, newRequests);
  allowedRequests.inc(newAllowed);
  httpsDecrypted.inc(newHttps);
  bandwidthSaved.inc(newBlocked * 50 * 1024);
  
  const latency = Math.random() * 0.002; // 0-2ms
  requestLatency.observe(latency);
  
}, 5000);

// API 路由

// 獲取統計數據
app.get('/api/stats', (req, res) => {
  res.json({
    totalRequests: stats.totalRequests,
    blockedAds: stats.blockedAds,
    allowedRequests: stats.allowedRequests,
    httpsDecrypted: stats.httpsDecrypted,
    bandwidthSaved: Math.round(stats.bandwidthSaved / 1024 / 1024), // MB
    blockRate: stats.totalRequests > 0 
      ? Math.round((stats.blockedAds / stats.totalRequests) * 100) 
      : 0,
    avgLatency: (Math.random() * 2).toFixed(2) + 'ms'
  });
});

// 獲取最近攔截記錄
app.get('/api/recent-blocks', (req, res) => {
  res.json(stats.recentBlocks.slice(0, 10));
});

// 獲取廣告規則統計
app.get('/api/rules', (req, res) => {
  res.json({
    blacklist: 150,
    whitelist: 50,
    keywordFilters: 200,
    httpsDecryption: 'enabled'
  });
});

// Prometheus 指標端點
app.get('/metrics', async (req, res) => {
  res.set('Content-Type', register.contentType);
  res.end(await register.metrics());
});

// 健康檢查
app.get('/health', (req, res) => {
  res.json({ status: 'healthy', timestamp: new Date().toISOString() });
});

// 啟動服務器
app.listen(PORT, () => {
  console.log(`🚀 Vwire Dashboard API running on port ${PORT}`);
  console.log(`📊 Metrics endpoint: http://localhost:${PORT}/metrics`);
  console.log(`🏥 Health check: http://localhost:${PORT}/health`);
});
