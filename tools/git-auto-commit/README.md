# Git Auto-Commit Tool

一個使用 Rust 開發的智慧型 Git 自動 commit 工具，透過 **LLM CLI** (預設使用 Gemini) 根據 staged 的變更自動產生 commit 訊息和分支名稱建議。

## 功能特色

- 🤖 使用 **LLM CLI** 智慧分析 `git diff --staged` 的變更內容
- 💬 AI 生成 3 個精準的繁體中文 commit 訊息建議
- 🌿 AI 生成 3 個符合規範的分支名稱建議
- 🎯 互動式選單介面，方便選擇
- ✨ 支援自訂 commit 訊息和分支名稱
- 🎨 美觀的命令列介面（使用色彩標示）
- 🦀 使用 Rust 開發，執行快速且安全
- ⚙️ 支援透過設定檔自訂 LLM CLI 指令和參數
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

### 必要：安裝 Gemini CLI

工具預設使用 `gemini` CLI 工具生成建議。請先安裝 Gemini CLI：

```bash
# 安裝 Gemini CLI（範例，請依實際情況調整）
# 詳細安裝方式請參考 Gemini CLI 官方文件
```

### 選用：自訂 LLM CLI 設定

工具支援透過設定檔自訂 LLM CLI 的呼叫方式。

#### 預設設定

若無設定檔，工具使用以下預設值：

```bash
gemini -p "prompt" --model "gemini-2.5-flash" --yolo
```

#### 建立自訂設定檔

1. 建立設定目錄：

```bash
mkdir -p ~/.config/git-auto-commit
```

2. 複製範例設定檔：

```bash
cp config.toml.example ~/.config/git-auto-commit/config.toml
```

3. 編輯設定檔 `~/.config/git-auto-commit/config.toml`：

```toml
# LLM CLI 指令（預設：gemini）
command = "gemini"

# 提示參數標記（預設：-p）
prompt_flag = "-p"

# 模型參數標記（預設：--model）
model_flag = "--model"

# 模型名稱（預設：gemini-2.5-flash）
model = "gemini-2.5-flash"

# 額外參數（預設：["--yolo"]）
extra_args = ["--yolo"]
```

#### 使用其他 LLM CLI

您可以設定使用任何支援命令列的 LLM 工具：

```toml
# 範例：使用 OpenAI CLI
command = "openai"
prompt_flag = "--prompt"
model_flag = "--model"
model = "gpt-4"
extra_args = ["--temperature", "0.7"]
```

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
- **LLM 整合方式**：透過 CLI 工具（預設：Gemini CLI with gemini-2.5-flash 模型）
- **主要相依套件**：
  - `git2` - Git 函式庫
  - `dialoguer` - 互動式命令列介面
  - `colored` - 終端機色彩輸出
  - `chrono` - 日期時間處理
  - `anyhow` - 錯誤處理
  - `serde` / `toml` - 設定檔解析

## 常見問題

### Q: 沒有安裝 Gemini CLI 會怎樣？

A: 工具會顯示錯誤訊息並自動降級使用規則式建議（基於檔案類型和變更類型的簡單邏輯），但建議品質會較差。

### Q: 可以使用其他 LLM CLI 嗎？

A: 可以！只需建立設定檔 `~/.config/git-auto-commit/config.toml` 並指定您的 LLM CLI 指令、參數和模型。工具支援任何能透過命令列呼叫的 LLM 工具。

### Q: 設定檔的格式是什麼？

A: 使用 TOML 格式。請參考專案中的 `config.toml.example` 檔案作為範本。設定檔應放置在 `~/.config/git-auto-commit/config.toml`。

### Q: 預設指令是什麼？

A: 預設執行 `gemini -p "prompt" --model "gemini-2.5-flash" --yolo`。可透過設定檔自訂所有參數。

### Q: 建議的品質如何？

A: LLM 會根據實際的 diff 內容和檔案脈絡生成精準的建議，通常比規則式方法更準確且更符合實際變更內容。建議品質取決於您使用的 LLM 模型。

## 授權

ISC License