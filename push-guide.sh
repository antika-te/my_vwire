#!/bin/bash
# 手動推送指南腳本
# 由於當前環境认证限制，請在本地執行此腳本

set -e

echo "📦 Vwire 專案推送指南"
echo ""
echo "由於環境认证限制，請選擇以下其中一種方式推送："
echo ""

# 方案 1: 使用 GitHub Token
cat << 'EOF'
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
方案 1: 使用 GitHub Token (推薦)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. 生成 Token:
   訪問 https://github.com/settings/tokens
   生成一個有 repo 權限的 personal access token

2. 在環境中設置:
   export GITHUB_TOKEN=<你的 token>

3. 執行推送:
   cd /workspace/vwire-filter
   git remote set-url origin https://$GITHUB_TOKEN@github.com/antika-te/my_vwire.git
   git push -u origin master

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
EOF
echo ""

# 方案 2: 下載專案到本地推送
cat << 'EOF'
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
方案 2: 下載專案到本地推送
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. 下載專案:
   cd /workspace
   tar czf vwire-filter.tar.gz vwire-filter --exclude=vwire-filter/target

2. 將 vwire-filter.tar.gz 發送到你的本地機器

3. 在本地解壓並推送:
   tar xzf vwire-filter.tar.gz
   cd vwire-filter
   git remote add origin https://github.com/antika-te/my_vwire.git
   git push -u origin master

   或使用 SSH:
   git remote set-url origin git@github.com:antika-te/my_vwire.git
   git push -u origin master

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
EOF
echo ""

# 檢查當前狀態
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "當前 Git 狀態:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
git status --short
echo ""
echo "已提交的 commit:"
git log --oneline
echo ""
echo "遠程倉庫配置:"
git remote -v
