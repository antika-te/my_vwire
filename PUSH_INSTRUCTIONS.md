# GitHub 推送指南

由於環境的 Git credential helper 限制，無法直接推送到 GitHub。請選擇以下其中一種方案完成推送。

---

## ✅ 方案 1: 使用 GitHub Token (推薦)

### 步驟 1: 生成 Personal Access Token

1. 訪問 GitHub Token 設置頁面: https://github.com/settings/tokens
2. 點擊 "Generate new token" → "Generate new token (classic)"
3. 設置 Note: `Vwire Project`
4. 設置 Expiration: `90 days` 或更長
5. 勾選權限：**repo** (Full control of private repositories)
6. 點擊 "Generate token"
7. **複製生成的 token** (格式如：`ghp_xxxxxxxxxxxx`)

### 步驟 2: 在環境中推送

```bash
# 設置 token (替換 <YOUR_TOKEN> 為實際 token)
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx

# 配置遠程倉庫使用 token
cd /workspace/vwire-filter
git remote set-url origin https://$GITHUB_TOKEN@github.com/antika-te/my_vwire.git

# 推送
git push -u origin master
```

### 步驟 3: 驗證

訪問 https://github.com/antika-te/my_vwire 確認代碼已推送成功。

### 步驟 4: 清理敏感信息 (可選)

```bash
# 恢復為不含 token 的 URL
git remote set-url origin https://github.com/antika-te/my_vwire.git

# 清除環境變量
unset GITHUB_TOKEN
```

---

## ✅ 方案 2: 下載專案到本地推送

### 步驟 1: 打包專案

```bash
cd /workspace
tar czf vwire-filter.tar.gz vwire-filter --exclude=vwire-filter/target
```

### 步驟 2: 下載到本地

將 `vwire-filter.tar.gz` 發送到你的本地機器。

### 步驟 3: 本地解壓並推送

```bash
# 解壓
mkdir my_vwire
cd my_vwire
tar xzf /path/to/vwire-filter.tar.gz

# 進入專案目錄
cd vwire-filter

# 推送到 GitHub (HTTPS 方式)
git remote add origin https://github.com/antika-te/my_vwire.git
git push -u origin master

# 或使用 SSH 方式 (需配置 SSH key)
# git remote add origin git@github.com:antika-te/my_vwire.git
# git push -u origin master
```

---

## ✅ 方案 3: 使用 GitHub CLI

### 步驟 1: 安裝 gh

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install gh

# macOS
brew install gh

# Arch Linux
sudo pacman -S github-cli
```

### 步驟 2: 認證

```bash
gh auth login
```

按照提示完成 GitHub 認證。

### 步驟 3: 推送

```bash
cd /workspace/vwire-filter

# 如果已有 origin remote
git remote set-url origin https://github.com/antika-te/my_vwire.git

# 推送
git push -u origin master
```

---

## 驗證推送結果

推送完成後，訪問: https://github.com/antika-te/my_vwire

確認包含以下文件：

- ✅ `.gitignore`
- ✅ `BUILDING.md`
- ✅ `Cargo.lock`
- ✅ `Cargo.toml`
- ✅ `README.md`
- ✅ `ROADMAP.md`
- ✅ `STATUS.md`
- ✅ `build.rs`
- ✅ `build.sh`
- ✅ `ebpf/Cargo.toml`
- ✅ `ebpf/src/main.rs`
- ✅ `run.sh`
- ✅ `src/ads_filter.rs`
- ✅ `src/config.rs`
- ✅ `src/main.rs`

---

## 後續開發

推送完成後，你可以：

1. **在本地繼續開發階段 2**:
   - 編譯 eBPF XDP 程序
   - 實現 AF_XDP 零拷貝通道
   - 集成 HTTP 響應注入

2. **參考文檔**:
   - `ROADMAP.md` - 完整開發路線圖
   - `BUILDING.md` - eBPF 編譯指南
   - `STATUS.md` - 當前進度和測試結果

3. **協作開發**:
   - 創建 feature branch 進行開發
   - 使用 Pull Request 合併代碼
   - 設置 GitHub Actions CI/CD

---

## 常見問題

### Q: `remote origin already exists`

A: 這是正常的，表示遠程已配置。直接執行推送即可。

```bash
git remote set-url origin <新 URL>
git push -u origin master
```

### Q: `permission denied`

A: 檢查 token 權限或 SSH key 配置。確保有寫入權限。

### Q: `could not read Username`

A: 環境不支持交互式認證。請使用 token 方式或本地推送。

---

**專案狀態**: ✅ 已完成 PoC 原型，等待推送到 GitHub
**Commit**: `5277744` - feat: 完成 Vwire eBPF/XDP 廣告過濾系統 PoC 原型

✊ 準備好推送了！
