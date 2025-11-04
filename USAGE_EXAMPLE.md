# Git Auto-Commit 使用範例

## 基本使用流程

### 1. 準備工作：Stage 你的變更

```bash
# 查看目前的變更
git status

# 添加檔案到 staging area
git add .
# 或只添加特定檔案
git add README.md src/index.js
```

### 2. 執行 git-auto-commit

```bash
npx git-auto-commit
# 或者如果已經全域安裝
git-auto-commit
```

### 3. 互動式操作流程

工具會按照以下順序引導你：

#### 步驟 1: 選擇分支

工具會顯示：
- 當前分支（可選擇保持）
- 3 個智慧產生的分支名稱建議
- 自訂分支名稱選項

```
? 選擇分支： (Use arrow keys)
❯ 保持當前分支 (main)
  --- 建議的分支名稱 ---
  1. feature/update-20251104
  2. refactor/improve-code-20251104
  3. chore/maintenance-20251104
  ──────────────
  自訂分支名稱
```

**操作方式**：
- 使用 `↑` `↓` 方向鍵選擇
- 按 `Enter` 確認選擇
- 如果選擇「自訂分支名稱」，會提示輸入自訂名稱

#### 步驟 2: 選擇 Commit 訊息

工具會顯示：
- 3 個根據變更內容智慧產生的繁體中文 commit 訊息
- 自訂 commit 訊息選項

```
? 選擇 Commit 訊息： (Use arrow keys)
  --- 建議的 Commit 訊息 ---
❯ 1. 新增：添加新檔案
  2. 文檔：新增專案文檔
  3. 更新：更新專案檔案
  ──────────────
  自訂 Commit 訊息
```

**操作方式**：
- 使用 `↑` `↓` 方向鍵選擇
- 按 `Enter` 確認選擇
- 如果選擇「自訂 Commit 訊息」，會提示輸入自訂訊息

#### 步驟 3: 確認並執行

最後會顯示確認提示：

```
? 確認要 commit？
  訊息：新增：添加新檔案 (Y/n)
```

**操作方式**：
- 按 `Y` 或直接按 `Enter` 確認執行
- 按 `n` 取消操作

### 4. 完成！

成功後會顯示：

```
✓ Commit 成功！
  訊息：新增：添加新檔案
```

## 實際使用範例

### 範例 1: 新增功能檔案

```bash
$ echo "console.log('Hello');" > src/feature.js
$ git add src/feature.js
$ npx git-auto-commit

🚀 Git 自動 Commit 工具

當前分支：main

📝 Staged 檔案：
  - src/feature.js

? 選擇分支： 保持當前分支 (main)
? 選擇 Commit 訊息： 1. 新增：添加 src/feature.js
? 確認要 commit？ Yes

✓ Commit 成功！
  訊息：新增：添加 src/feature.js
```

### 範例 2: 更新文檔並切換分支

```bash
$ echo "# API Documentation" > docs/API.md
$ git add docs/API.md
$ npx git-auto-commit

🚀 Git 自動 Commit 工具

當前分支：main

📝 Staged 檔案：
  - docs/API.md

? 選擇分支： 1. docs/update-docs-20251104
✓ 已切換到新分支：docs/update-docs-20251104

? 選擇 Commit 訊息： 1. 文檔：新增專案文檔
? 確認要 commit？ Yes

✓ Commit 成功！
  訊息：文檔：新增專案文檔
```

### 範例 3: 使用自訂訊息

```bash
$ git add config.json
$ npx git-auto-commit

🚀 Git 自動 Commit 工具

當前分支：main

📝 Staged 檔案：
  - config.json

? 選擇分支： 保持當前分支 (main)
? 選擇 Commit 訊息： 自訂 Commit 訊息
? 請輸入自訂 Commit 訊息： 配置：更新 API 端點設定
? 確認要 commit？ Yes

✓ Commit 成功！
  訊息：配置：更新 API 端點設定
```

## Commit 訊息類型說明

工具會根據檔案類型和變更內容，自動產生適當的 commit 訊息前綴：

- **新增**：新增檔案時使用
- **刪除**：刪除檔案時使用
- **修復**：修正程式錯誤
- **優化**：改善程式效能
- **重構**：重構程式碼結構
- **文檔**：更新文檔檔案
- **配置**：更新設定檔
- **測試**：更新測試檔案
- **樣式**：調整介面樣式
- **更新**：一般性更新
- **維護**：日常維護

## 分支命名規範

工具會根據變更內容建議適當的分支名稱格式：

- `feature/` - 新功能開發
- `fix/` - 錯誤修復
- `docs/` - 文檔更新
- `config/` - 設定檔調整
- `test/` - 測試相關
- `refactor/` - 程式碼重構
- `chore/` - 日常維護

## 注意事項

1. **必須先 stage 變更**：執行工具前，必須先使用 `git add` 將變更加入 staging area
2. **自動產生的訊息僅供參考**：建議根據實際變更內容選擇或自訂更準確的訊息
3. **分支切換**：如果選擇切換分支，工具會自動建立新分支
4. **確認再執行**：最後一步會要求確認，可以在此檢查是否正確

## 故障排除

### 問題：顯示「沒有 staged 的檔案變更」

**解決方法**：
```bash
# 先檢查是否有變更
git status

# 添加變更到 staging area
git add <檔案名稱>
# 或
git add .
```

### 問題：切換分支失敗

**可能原因**：
- 分支名稱已存在
- 有未提交的變更

**解決方法**：
```bash
# 檢查現有分支
git branch -a

# 使用不同的分支名稱或選擇「自訂分支名稱」
```

### 問題：Commit 失敗

**可能原因**：
- Git 配置未設定
- Commit 訊息格式問題

**解決方法**：
```bash
# 設定 Git 使用者資訊
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

## 進階技巧

### 1. 結合 Git Hooks

可以將此工具整合到 Git hooks 中，實現更自動化的工作流程。

### 2. 自訂訊息模板

如果團隊有特定的 commit 訊息格式，可以選擇「自訂 Commit 訊息」並按照團隊規範輸入。

### 3. 快速操作

熟悉工具後，可以快速使用方向鍵和 Enter 鍵完成選擇，提升效率。

## 回饋與貢獻

如果您有任何問題、建議或想要貢獻程式碼，歡迎在 GitHub 上開啟 Issue 或 Pull Request！
