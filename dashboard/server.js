// Vwire Dashboard Backend API v0.5.0
// 提供配置管理和監控 API

const express = require('express');
const cors = require('cors');
const promClient = require('prom-client');
const path = require('path');
const fs = require('fs');

const app = express();
const PORT = process.env.PORT || 3001;

// 啟用 CORS 和 JSON
app.use(cors());
app.use(express.json());

// 提供靜態文件
app.use('/', express.static(path.join(__dirname, '.')));

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

register.registerMetric(totalRequests);
register.registerMetric(blockedAds);

// 配置存儲
let config = {
  rules: [
    { domain: 'googleadservices.com', type: 'ads', hits: 1234 },
    { domain: 'doubleclick.net', type: 'ads', hits: 987 },
    { domain: 'ads.facebook.com', type: 'social', hits: 756 },
  ],
  whitelist: [
    { domain: 'api.example.com', note: '內部 API' },
  ],
  settings: {
    interface: 'eth0',
    xdpMode: 'native',
    logLevel: 'info',
    enableHttps: true,
    enableBackend: true
  }
};

// API 路由

// 獲取統計數據
app.get('/api/stats', (req, res) => {
  totalRequests.inc({ type: 'http' }, 1);
  
  res.json({
    totalRequests: Math.floor(Math.random() * 100000),
    blockedAds: Math.floor(Math.random() * 50000),
    allowedRequests: Math.floor(Math.random() * 50000),
    httpsDecrypted: Math.floor(Math.random() * 30000),
    bandwidthSaved: Math.floor(Math.random() * 500),
    blockRate: Math.floor(Math.random() * 30 + 50),
    avgLatency: (Math.random() * 2).toFixed(2) + 'ms'
  });
});

// 獲取廣告規則
app.get('/api/rules', (req, res) => {
  res.json(config.rules);
});

// 添加廣告規則
app.post('/api/rules', (req, res) => {
  const { domain, type } = req.body;
  config.rules.push({ domain, type: type || 'ads', hits: 0 });
  res.json({ success: true, domain });
});

// 刪除廣告規則
app.delete('/api/rules/:index', (req, res) => {
  const index = parseInt(req.params.index);
  if (config.rules[index]) {
    config.rules.splice(index, 1);
    res.json({ success: true });
  } else {
    res.status(404).json({ error: 'Rule not found' });
  }
});

// 獲取白名單
app.get('/api/whitelist', (req, res) => {
  res.json(config.whitelist);
});

// 添加白名單
app.post('/api/whitelist', (req, res) => {
  const { domain, note } = req.body;
  config.whitelist.push({ domain, note: note || '' });
  res.json({ success: true });
});

// 刪除白名單
app.delete('/api/whitelist/:index', (req, res) => {
  const index = parseInt(req.params.index);
  if (config.whitelist[index]) {
    config.whitelist.splice(index, 1);
    res.json({ success: true });
  } else {
    res.status(404).json({ error: 'Not found' });
  }
});

// 保存配置
app.post('/api/config', (req, res) => {
  const { rules, whitelist } = req.body;
  if (rules) config.rules = rules;
  if (whitelist) config.whitelist = whitelist;
  res.json({ success: true });
});

// 獲取設置
app.get('/api/settings', (req, res) => {
  res.json(config.settings);
});

// 保存設置
app.post('/api/settings', (req, res) => {
  config.settings = { ...config.settings, ...req.body };
  res.json({ success: true });
});

// 健康檢查
app.get('/health', (req, res) => {
  res.json({ 
    status: 'healthy', 
    timestamp: new Date().toISOString(),
    version: '0.5.0'
  });
});

// Prometheus 指標
app.get('/metrics', async (req, res) => {
  res.set('Content-Type', register.contentType);
  res.end(await register.metrics());
});

// 啟動服務器
app.listen(PORT, () => {
  console.log(`🚀 Vwire Dashboard API v0.5.0 running on port ${PORT}`);
  console.log(`📊 Metrics: http://localhost:${PORT}/metrics`);
  console.log(`🏥 Health: http://localhost:${PORT}/health`);
  console.log(`⚙️ Admin UI: http://localhost:${PORT}/admin.html`);
});
