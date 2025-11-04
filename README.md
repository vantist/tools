# Tools

一個 Rust 工具集合，包含多個實用的命令列工具。

## 工具列表

### Git Auto-Commit
位於 [`tools/git-auto-commit/`](tools/git-auto-commit/)

一個使用 Rust 開發的智慧型 Git 自動 commit 工具，透過 LLM CLI (預設使用 Gemini) 根據 staged 的變更自動產生 commit 訊息和分支名稱建議。

**主要功能：**
- 🤖 使用 LLM CLI 智慧分析 git diff 內容
- 💬 AI 生成 3 個精準的繁體中文 commit 訊息建議
- 🌿 AI 生成 3 個符合規範的分支名稱建議
- 🎯 互動式選單介面
- ⚙️ 支援透過設定檔自訂 LLM CLI 指令和參數

[查看詳細說明 →](tools/git-auto-commit/README.md)

### Example Tool
位於 [`tools/example-tool/`](tools/example-tool/)

範例工具，展示專案結構。

## 安裝方式

### 快速安裝（推薦）

使用自動建置與安裝腳本，一鍵安裝所有工具到 `~/bin/`：

#### macOS / Linux

```bash
# 克隆專案
git clone https://github.com/vantist/tools.git
cd tools

# 建置並安裝所有工具
./build-and-install.sh
```

腳本會：

- 🔨 自動建置所有工具（release 版本）
- 📁 建立 `~/bin/` 目錄（如果不存在）
- 🔗 建立符號連結（symbolic links）將所有執行檔連結至 `~/bin/`

**注意：**

- 腳本使用符號連結，因此請保留 `target/release/` 目錄
- 請確保 `~/bin` 已加入 PATH 環境變數。如果尚未設定，請在 `~/.bashrc` 或 `~/.zshrc` 中加入：

```bash
export PATH="$HOME/bin:$PATH"
```

#### Windows

```powershell
# 克隆專案
git clone https://github.com/vantist/tools.git
cd tools

# 建置並安裝所有工具
.\build-and-install.ps1
```

腳本會：

- 🔨 自動建置所有工具（release 版本）
- 📁 建立 `%USERPROFILE%\bin\` 目錄（如果不存在）
- 📋 複製所有執行檔（.exe）至 `%USERPROFILE%\bin\`

**注意：**

- 請確保 `%USERPROFILE%\bin` 已加入 PATH 環境變數
- 可以透過以下 PowerShell 命令新增到使用者環境變數：

```powershell
$env:Path += ";$env:USERPROFILE\bin"
[Environment]::SetEnvironmentVariable("Path", $env:Path, [System.EnvironmentVariableTarget]::User)
```

或手動設定：

1. 開啟「系統內容」→「進階系統設定」→「環境變數」
2. 在「使用者變數」中編輯 `Path`
3. 新增：`%USERPROFILE%\bin`

### 從原始碼編譯

```bash
# 克隆專案
git clone https://github.com/vantist/tools.git
cd tools

# 編譯所有工具
cargo build --release

# 或編譯特定工具
cd tools/git-auto-commit
cargo build --release
```

編譯完成後，可執行檔位於 `target/release/` 目錄下。

### 安裝 Rust

如果還沒有安裝 Rust，可以使用以下命令安裝：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 專案結構

```text
tools/
├── Cargo.toml              # Workspace 設定檔
├── tools/
│   ├── git-auto-commit/   # Git 自動 commit 工具
│   └── example-tool/      # 範例工具
└── README.md              # 本檔案
```

## 開發

```bash
# 在 workspace 根目錄執行所有測試
cargo test

# 檢查所有工具
cargo check --workspace

# 建立所有 release 版本
cargo build --release --workspace
```

## 授權

MIT License
