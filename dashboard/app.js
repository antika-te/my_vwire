// Vwire 配置管理前端

const API_BASE = '/api';

// 模擬數據
const mockRules = [
    { domain: 'googleadservices.com', type: 'ads', hits: 1234 },
    { domain: 'doubleclick.net', type: 'ads', hits: 987 },
    { domain: 'ads.facebook.com', type: 'social', hits: 756 },
    { domain: 'analytics.google.com', type: 'analytics', hits: 654 },
    { domain: 'amazon-adsystem.com', type: 'ads', hits: 543 },
];

const mockWhitelist = [
    { domain: 'api.example.com', note: '內部 API' },
    { domain: 'fonts.googleapis.com', note: 'Google Fonts' },
];

// 初始化
document.addEventListener('DOMContentLoaded', () => {
    loadRules();
    loadWhitelist();
    loadStats();
    setupTabs();
});

// 標籤頁切換
function setupTabs() {
    document.querySelectorAll('.tab').forEach(tab => {
        tab.addEventListener('click', () => {
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content > div').forEach(c => c.style.display = 'none');
            
            tab.classList.add('active');
            document.getElementById(`${tab.dataset.tab}-tab`).style.display = 'block';
        });
    });
}

// 加載規則
async function loadRules() {
    try {
        const response = await fetch(`${API_BASE}/rules`);
        const rules = await response.json();
        renderRules(rules || mockRules);
    } catch (e) {
        renderRules(mockRules);
    }
}

// 渲染規則列表
function renderRules(rules) {
    const list = document.getElementById('rules-list');
    list.innerHTML = rules.map((rule, i) => `
        <div class="rule-item">
            <div>
                <span class="rule-domain">${rule.domain}</span>
                <small style="color: #666; margin-left: 10px;">${rule.type}</small>
                <small style="color: #999; margin-left: 10px;">命中：${rule.hits}</small>
            </div>
            <button onclick="removeRule(${i})" style="padding: 0.25rem 0.5rem; background: #ef4444;">刪除</button>
        </div>
    `).join('');
    
    document.getElementById('rule-count').textContent = rules.length;
    document.getElementById('total-rules').textContent = rules.length;
}

// 添加規則
async function addRule() {
    const input = document.getElementById('new-rule');
    const domain = input.value.trim();
    
    if (!domain) return;
    
    try {
        await fetch(`${API_BASE}/rules`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ domain, type: 'ads' })
        });
        input.value = '';
        loadRules();
    } catch (e) {
        mockRules.push({ domain, type: 'ads', hits: 0 });
        input.value = '';
        renderRules(mockRules);
    }
}

// 刪除規則
async function removeRule(index) {
    if (!confirm('確定刪除此規則？')) return;
    
    try {
        await fetch(`${API_BASE}/rules/${index}`, { method: 'DELETE' });
        loadRules();
    } catch (e) {
        mockRules.splice(index, 1);
        renderRules(mockRules);
    }
}

// 加載白名單
async function loadWhitelist() {
    try {
        const response = await fetch(`${API_BASE}/whitelist`);
        const whitelist = await response.json();
        renderWhitelist(whitelist || mockWhitelist);
    } catch (e) {
        renderWhitelist(mockWhitelist);
    }
}

// 渲染白名單
function renderWhitelist(whitelist) {
    const list = document.getElementById('whitelist-list');
    list.innerHTML = whitelist.map((item, i) => `
        <div class="rule-item">
            <div>
                <span class="rule-domain">${item.domain}</span>
                <small style="color: #666; margin-left: 10px;">${item.note || ''}</small>
            </div>
            <button onclick="removeWhitelist(${i})" style="padding: 0.25rem 0.5rem; background: #ef4444;">刪除</button>
        </div>
    `).join('');
    
    document.getElementById('whitelist-count').textContent = whitelist.length;
    document.getElementById('whitelist-rules').textContent = whitelist.length;
}

// 添加白名單
async function addWhitelist() {
    const input = document.getElementById('new-whitelist');
    const domain = input.value.trim();
    
    if (!domain) return;
    
    try {
        await fetch(`${API_BASE}/whitelist`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ domain })
        });
        input.value = '';
        loadWhitelist();
    } catch (e) {
        mockWhitelist.push({ domain, note: '' });
        input.value = '';
        renderWhitelist(mockWhitelist);
    }
}

// 刪除白名單
function removeWhitelist(index) {
    if (!confirm('確定刪除此白名單？')) return;
    mockWhitelist.splice(index, 1);
    renderWhitelist(mockWhitelist);
}

// 加載統計
async function loadStats() {
    try {
        const response = await fetch(`${API_BASE}/stats`);
        const stats = await response.json();
        document.getElementById('blocked-today').textContent = stats.blockedAds || 0;
    } catch (e) {
        // 使用模擬數據
    }
    
    // 模擬運行時間
    const uptime = Math.floor(Math.random() * 24) + 1;
    document.getElementById('uptime').textContent = `${uptime}h`;
}

// 保存配置
async function saveConfig() {
    try {
        await fetch(`${API_BASE}/config`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                rules: mockRules,
                whitelist: mockWhitelist
            })
        });
        alert('配置已保存！');
    } catch (e) {
        alert('配置保存成功（演示模式）');
    }
}

// 重置配置
function resetConfig() {
    if (!confirm('確定重置為默認配置？')) return;
    mockRules.length = 0;
    mockWhitelist.length = 0;
    loadRules();
    loadWhitelist();
}

// 保存設置
async function saveSettings() {
    const settings = {
        interface: document.getElementById('interface').value,
        xdpMode: document.getElementById('xdp-mode').value,
        logLevel: document.getElementById('log-level').value,
        enableHttps: document.getElementById('enable-https').checked,
        enableBackend: document.getElementById('enable-backend').checked
    };
    
    try {
        await fetch(`${API_BASE}/settings`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(settings)
        });
        alert('設置已保存！');
    } catch (e) {
        alert('設置保存成功（演示模式）');
    }
}

// 刷新日誌
function refreshLogs() {
    const viewer = document.getElementById('log-viewer');
    const now = new Date().toLocaleString();
    viewer.value += `\n${now} INFO  日誌已刷新`;
    viewer.scrollTop = viewer.scrollHeight;
}

// 下載日誌
function downloadLogs() {
    const viewer = document.getElementById('log-viewer');
    const blob = new Blob([viewer.value], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `vwire-logs-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
}
