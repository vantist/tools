# Git Auto-Commit Tool

一個使用 Rust 開發的智慧型 Git 自動 commit 工具，透過 **Gemini LLM** 根據 staged 的變更自動產生 commit 訊息和分支名稱建議。

## 功能特色

- 🤖 使用 **Gemini LLM** 智慧分析 `git diff --staged` 的變更內容
- 💬 AI 生成 3 個精準的繁體中文 commit 訊息建議
- 🌿 AI 生成 3 個符合規範的分支名稱建議
- 🎯 互動式選單介面，方便選擇
- ✨ 支援自訂 commit 訊息和分支名稱
- 🎨 美觀的命令列介面（使用色彩標示）
- 🦀 使用 Rust 開發，執行快速且安全
- 🔄 LLM 失敗時自動降級到規則式建議

## 安裝方式

### 從原始碼編譯

```bash
# 克隆專案
git clone https://github.com/vantist/tools.git
cd tools

# 編譯（需要先安裝 Rust）
cargo build --release

# 可執行檔位於 target/release/git-auto-commit
# 可以將它複製到 PATH 中的目錄，例如：
sudo cp target/release/git-auto-commit /usr/local/bin/
```

### 安裝 Rust

如果還沒有安裝 Rust，可以使用以下命令安裝：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 設定

### 必要：設定 Gemini API Key

工具使用 Google Gemini API 生成建議，需要先設定 API Key：

1. 前往 [Google AI Studio](https://makersuite.google.com/app/apikey) 取得免費的 API Key
2. 設定環境變數：

```bash
export GEMINI_API_KEY="your-api-key-here"
```

建議將此設定加入 `~/.bashrc` 或 `~/.zshrc` 以便永久使用：

```bash
echo 'export GEMINI_API_KEY="your-api-key-here"' >> ~/.bashrc
source ~/.bashrc
```

### 選用：預設使用 Gemini Pro

工具預設使用 `gemini-pro` 模型，此為免費模型，適合日常使用。

## 使用方式

1. 先將要 commit 的檔案加入 staging area：
```bash
git add .
# 或
git add <檔案名稱>
```

2. 執行 git-auto-commit：
```bash
git-auto-commit
```

工具會自動使用 Gemini LLM 分析您的變更並生成建議。

3. 跟隨互動式選單操作：
   - 選擇是否要切換到新分支（或保持當前分支）
   - 選擇 AI 生成的 commit 訊息（或自訂）
   - 確認後執行 commit

## 使用範例

```bash
# 1. 修改一些檔案
$ echo "test" > test.txt

# 2. 加入 staging area
$ git add test.txt

# 3. 執行自動 commit
$ git-auto-commit

🚀 Git 自動 Commit 工具

當前分支：main

📝 Staged 檔案：
  - test.txt

🤖 正在使用 LLM 生成分支名稱建議...
🤖 正在使用 LLM 生成 commit 訊息建議...

? 選擇分支： 
  > 保持當前分支 (main)
    --- 建議的分支名稱 ---
    1. feature/add-test-file-20251104
    2. docs/update-test-content-20251104
    3. chore/add-test-resource-20251104
    ──────────────
    自訂分支名稱

? 選擇 Commit 訊息： 
    --- 建議的 Commit 訊息 ---
  > 1. 新增：添加測試檔案 test.txt
    2. 文檔：新增測試用文檔檔案
    3. 維護：添加專案測試資源
    ──────────────
    自訂 Commit 訊息

? 確認要 commit？
  訊息：新增：添加測試檔案 test.txt › yes

✓ Commit 成功！
  訊息：新增：添加測試檔案 test.txt
```

## 開發

```bash
# 執行開發版本
cargo run

# 執行測試
cargo test

# 建立 release 版本
cargo build --release
```

## 技術細節

- **程式語言**：Rust
- **AI 模型**：Google Gemini Pro（預設）
- **主要相依套件**：
  - `git2` - Git 函式庫
  - `dialoguer` - 互動式命令列介面
  - `colored` - 終端機色彩輸出
  - `chrono` - 日期時間處理
  - `anyhow` - 錯誤處理
  - `reqwest` - HTTP 客戶端（用於 API 請求）
  - `serde` / `serde_json` - JSON 序列化

## 常見問題

### Q: 沒有設定 GEMINI_API_KEY 會怎樣？

A: 工具會顯示錯誤訊息並自動降級使用規則式建議（基於檔案類型和變更類型的簡單邏輯），但建議品質會較差。

### Q: Gemini API 是否免費？

A: 是的，Gemini Pro 提供免費額度，每分鐘 60 次請求，對於日常使用綽綽有餘。詳見 [Google AI 定價](https://ai.google.dev/pricing)。

### Q: 可以使用其他 LLM 嗎？

A: 目前僅支援 Gemini，但程式碼設計上容易擴展。歡迎提交 PR 添加其他 LLM 支援（如 OpenAI、Claude 等）。

### Q: 建議的品質如何？

A: Gemini Pro 會根據實際的 diff 內容和檔案脈絡生成精準的建議，通常比規則式方法更準確且更符合實際變更內容。

## 授權

ISC License