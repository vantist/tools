use anyhow::{Context, Result};
use chrono::Local;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use git2::{Repository, StatusOptions};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<()> {
    println!("\n{}\n", "🚀 Git 自動 Commit 工具".cyan().bold());

    // 檢查是否在 git repository 中
    // 使用當前工作目錄而非執行檔所在目錄
    let current_dir = env::current_dir().context("無法取得當前目錄")?;
    let repo = Repository::open(&current_dir).context("✗ 錯誤：當前目錄不是 Git repository")?;

    // 取得當前分支
    let current_branch = get_current_branch(&repo)?;
    println!("{}", format!("當前分支：{}\n", current_branch).dimmed());

    // 檢查 staged 變更
    let staged_files = get_staged_files(&repo)?;
    if staged_files.is_empty() {
        println!(
            "{}",
            "⚠️  沒有 staged 的檔案變更，請先使用 git add 加入檔案"
                .yellow()
        );
        std::process::exit(1);
    }

    // 顯示 staged 檔案
    println!("{}", "📝 Staged 檔案：".blue());
    for file in &staged_files {
        println!("{}", format!("  - {}", file).dimmed());
    }
    println!();

    // 取得 diff 內容用於分析
    let diff_content = get_staged_diff(&repo)?;

    // 載入設定（只載入一次）
    let config = load_llm_config();

    // 生成建議（單次 LLM 請求）
    let suggestions = generate_suggestions(&diff_content, &staged_files, &config);

    // 詢問是否要切換分支
    let branch_choice = select_branch(&current_branch, &suggestions.branch_names)?;

    // 處理分支切換
    if let Some(new_branch) = branch_choice {
        switch_branch(&new_branch)?;
    }

    println!();

    // 詢問 commit 訊息（內含預覽和確認循環）
    let commit_message = select_commit_message(&suggestions.commit_messages)?;

    // 執行 commit
    commit_changes(&commit_message)?;

    println!();
    Ok(())
}

/// 取得當前分支名稱
fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head()?;
    let branch_name = head
        .shorthand()
        .unwrap_or("main")
        .to_string();
    Ok(branch_name)
}

/// 取得 staged 的檔案列表
fn get_staged_files(repo: &Repository) -> Result<Vec<String>> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(false);
    
    let statuses = repo.statuses(Some(&mut opts))?;
    let mut staged_files = Vec::new();

    for entry in statuses.iter() {
        let status = entry.status();
        if status.is_index_new()
            || status.is_index_modified()
            || status.is_index_deleted()
            || status.is_index_renamed()
            || status.is_index_typechange()
        {
            if let Some(path) = entry.path() {
                staged_files.push(path.to_string());
            }
        }
    }

    Ok(staged_files)
}

/// 取得 staged 的 diff 內容（優化版，減少 token 使用但保留關鍵資訊）
fn get_staged_diff(_repo: &Repository) -> Result<String> {
    // 優化參數說明：
    // --inter-hunk-context=1: 減少 hunk 之間的空白行
    // --ignore-space-change: 忽略空白變更（減少雜訊）
    // --ignore-blank-lines: 忽略空白行變更
    // --no-prefix: 移除 a/ 和 b/ 前綴（節省 token）
    // --no-color: 確保沒有 ANSI 顏色碼
    let output = Command::new("git")
        .args(&[
            "diff",
            "--staged",
            "--inter-hunk-context=1",
            "--ignore-space-change",
            "--ignore-blank-lines",
            "--no-prefix",
            "--no-color"
        ])
        .output()
        .context("無法執行 git diff")?;

    if !output.status.success() {
        anyhow::bail!("git diff 執行失敗");
    }

    let diff = String::from_utf8_lossy(&output.stdout).to_string();
    
    Ok(diff)
}

/// 取得檔案的簡要資訊
fn get_file_summary(files: &[String]) -> String {
    let mut summary = String::new();
    
    for file in files {
        let path = std::path::Path::new(file);
        
        // 判斷檔案類型
        let file_type = if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("rs") => "Rust 程式碼",
                Some("js") | Some("ts") => "JavaScript/TypeScript",
                Some("py") => "Python 程式碼",
                Some("java") => "Java 程式碼",
                Some("go") => "Go 程式碼",
                Some("md") => "Markdown 文檔",
                Some("toml") | Some("yaml") | Some("yml") | Some("json") => "設定檔",
                Some("html") | Some("css") => "前端檔案",
                _ => "其他檔案",
            }
        } else {
            "無副檔名"
        };
        
        summary.push_str(&format!("- {}: {}\n", file, file_type));
    }
    
    summary
}

/// LLM 建議結果
#[derive(Debug, Clone)]
struct GitSuggestions {
    branch_names: Vec<String>,
    commit_messages: Vec<String>,
}

/// LLM CLI 設定
#[derive(Debug, Deserialize, Serialize, Clone)]
struct LlmConfig {
    /// LLM CLI 指令（例如：gemini）
    #[serde(default = "default_command")]
    command: String,
    /// 提示參數標記（例如：-p）
    #[serde(default = "default_prompt_flag")]
    prompt_flag: String,
    /// 模型參數標記（例如：--model）
    #[serde(default = "default_model_flag")]
    model_flag: String,
    /// 模型名稱（例如：gemini-2.5-flash）
    #[serde(default = "default_model")]
    model: String,
    /// 額外參數（例如：--yolo）
    #[serde(default = "default_extra_args")]
    extra_args: Vec<String>,
    /// 合併的提示詞模板
    #[serde(default = "default_combined_prompt")]
    combined_prompt: String,
}

fn default_command() -> String {
    "gemini".to_string()
}

fn default_prompt_flag() -> String {
    "-p".to_string()
}

fn default_model_flag() -> String {
    "--model".to_string()
}

fn default_model() -> String {
    "gemini-2.5-flash".to_string()
}

fn default_extra_args() -> Vec<String> {
    vec![]
}

fn default_combined_prompt() -> String {
    r#"你是一個 Git 專家。請根據以下資訊，生成分支名稱和 commit 訊息建議。

變更統計：
{stats}

檔案列表與類型：
{file_summary}

詳細變更（Git diff with context）：
```
{diff}
```

Determine the best branch naming prefixes.

Here are the prefixes you can choose from:

- feature/: For new features (e.g., feature/add-login-page, feat/add-login-page)
- bugfix/: For bug fixes (e.g., bugfix/fix-header-bug, fix/header-bug)
- hotfix/: For urgent fixes (e.g., hotfix/security-patch)
- release/: For branches preparing a release (e.g., release/v1.2.0)
- chore/: For non-code tasks like dependency, docs updates (e.g., chore/update-dependencies)

Determine the best label for the commit.

Here are the labels you can choose from:

- build: Changes that affect the build system or external dependencies (example scopes: gulp, broccoli, npm)
- chore: Updating libraries, copyrights, or other repo settings, includes updating dependencies.
- ci: Changes to our CI configuration files and scripts (example scopes: Travis, Circle, GitHub Actions)
- docs: Non-code changes, such as fixing typos or adding new documentation (example scopes: Markdown files)
- feat: A commit of the type feat introduces a new feature to the codebase
- fix: A commit of the type fix patches a bug in your codebase
- perf: A code change that improves performance
- refactor: A code change that neither fixes a bug nor adds a feature
- style: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc.)
- test: Adding missing tests or correcting existing tests

請按照以下格式回覆：

[BRANCHES]
feature/example-feature
fix/example-bug
chore/example-task

[COMMITS]
feat: 新增使用者登入功能

實作完整的使用者登入流程，包含密碼驗證與 session 管理。


fix: 修正資料庫連線錯誤

修正了在高並發情況下資料庫連線池耗盡的問題。


chore: 更新專案依賴套件

更新所有依賴套件至最新穩定版本，提升安全性。

要求：
1. 仔細分析 diff 的完整上下文，理解變更的真實意圖
2. [BRANCHES] 區塊包含 3 個分支名稱建議，格式為「type/description」
   - type 可選：請依據 naming prefixes 選擇最合適的類型
   - description 使用英文小寫，單字之間用連字號 - 連接，不超過 30 字元
3. [COMMITS] 區塊包含 3 個 commit 訊息建議
   - **重要**：每個 commit 訊息必須以「type:」開頭（type 為英文）
   - 第一行格式：「type: 簡短描述」，type 使用英文，描述使用繁體中文
   - type 可選：請依據上述 labels 選擇最合適的類型
   - 描述要精確反映實際變更內容，不超過 50 字
   - 並補充說明，在第二行之後使用繁體中文詳細說明（限 5 行內）
   - **重要**：每個 commit 訊息之間必須用空行分隔
4. 不要使用 markdown 格式，不要編號
5. 善用函數名稱、變數名稱等上下文資訊來理解變更目的
6. 確保每個 commit 訊息都是完整且獨立的，不要將說明文字誤認為獨立的 commit"#
        .to_string()
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            command: default_command(),
            prompt_flag: default_prompt_flag(),
            model_flag: default_model_flag(),
            model: default_model(),
            extra_args: default_extra_args(),
            combined_prompt: default_combined_prompt(),
        }
    }
}

/// 取得設定檔路徑
fn get_config_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("git-auto-commit").join("config.toml")
}

/// 載入 LLM 設定
fn load_llm_config() -> LlmConfig {
    let config_path = get_config_path();
    
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match toml::from_str::<LlmConfig>(&content) {
                    Ok(config) => {
                        println!("{}", format!("📝 已載入設定檔：{}", config_path.display()).dimmed());
                        return config;
                    }
                    Err(e) => {
                        println!("{}", format!("⚠️  設定檔格式錯誤：{}，使用預設設定", e).yellow());
                    }
                }
            }
            Err(e) => {
                println!("{}", format!("⚠️  無法讀取設定檔：{}，使用預設設定", e).yellow());
            }
        }
    }
    
    LlmConfig::default()
}

/// 使用 LLM CLI 生成建議
fn call_llm_cli(prompt: &str, config: &LlmConfig) -> Result<String> {
    
    // 建立指令
    let mut cmd = Command::new(&config.command);
    
    // 添加提示參數
    cmd.arg(&config.prompt_flag).arg(prompt);
    
    // 添加模型參數
    cmd.arg(&config.model_flag).arg(&config.model);
    
    // 添加額外參數
    for arg in &config.extra_args {
        cmd.arg(arg);
    }
    
    // 執行指令
    let output = cmd
        .output()
        .context(format!("無法執行 {} 指令，請確認已安裝 {} CLI 工具", config.command, config.command))?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("{} 執行失敗：{}", config.command, error);
    }
    
    let response = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(response.trim().to_string())
}

/// 生成分支和 commit 建議（使用 LLM，單次請求）
fn generate_suggestions(diff: &str, files: &[String], config: &LlmConfig) -> GitSuggestions {
    println!("{}", "🤖 正在使用 LLM 生成建議...".dimmed());
    
    // 增加檔案類型摘要，提供更多上下文
    let file_summary = get_file_summary(files);
    
    // 計算 diff 的統計資訊
    let stats = get_diff_stats(diff);
    
    // 根據 diff 大小動態調整限制（增加到 8000 字元以保留更多上下文）
    let diff_preview = if diff.len() > 8000 {
        // 如果超過限制，優先保留前面和後面的部分
        let front = &diff[..4000];
        let back_start = diff.len().saturating_sub(4000);
        let back = &diff[back_start..];
        format!("{}\n\n... (中間省略) ...\n\n{}", front, back)
    } else {
        diff.to_string()
    };

    let files_list = files.join(", ");
    
    // 使用合併的提示詞模板，加入更多上下文資訊
    let prompt = config.combined_prompt
        .replace("{files}", &files_list)
        .replace("{file_summary}", &file_summary)
        .replace("{stats}", &stats)
        .replace("{diff}", &diff_preview);

    match call_llm_cli(&prompt, config) {
        Ok(response) => {
            // 解析 LLM 回應
            if let Some(suggestions) = parse_llm_response(&response) {
                return suggestions;
            }
        }
        Err(e) => {
            println!("{}", format!("⚠️  LLM 生成失敗：{}", e).yellow());
            println!("{}", "使用備用建議...".dimmed());
        }
    }

    // 備用建議（如果 LLM 失敗）
    GitSuggestions {
        branch_names: generate_fallback_branch_suggestions(files),
        commit_messages: generate_fallback_commit_suggestions(diff, files),
    }
}

/// 取得 diff 的統計資訊
fn get_diff_stats(diff: &str) -> String {
    let mut additions = 0;
    let mut deletions = 0;
    let mut files_changed = 0;
    
    for line in diff.lines() {
        if line.starts_with("+++") || line.starts_with("---") {
            if !line.contains("/dev/null") {
                files_changed += 1;
            }
        } else if line.starts_with('+') && !line.starts_with("+++") {
            additions += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            deletions += 1;
        }
    }
    
    // 修正檔案數量（每個檔案會有 +++ 和 --- 兩行）
    files_changed = files_changed / 2;
    
    format!(
        "{} 個檔案變更，新增 {} 行，刪除 {} 行",
        files_changed, additions, deletions
    )
}

/// 解析 LLM 回應，提取分支名稱和 commit 訊息
fn parse_llm_response(response: &str) -> Option<GitSuggestions> {
    let mut branch_names = Vec::new();
    let mut commit_messages = Vec::new();
    
    // 找到 [BRANCHES] 和 [COMMITS] 區塊
    let branches_start = response.find("[BRANCHES]")?;
    let commits_start = response.find("[COMMITS]")?;
    
    // 提取分支名稱區塊
    let branches_section = &response[branches_start + 10..commits_start];
    for line in branches_section.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.contains('/') {
            branch_names.push(trimmed.to_string());
        }
    }
    
    // 提取 commit 訊息區塊
    let commits_section = &response[commits_start + 9..];
    
    // 使用更智能的方式解析 commit 訊息
    // 符合 "word:" 格式的行被視為新 commit 的開始（允許任何類型）
    let mut current_commit = String::new();
    
    for line in commits_section.lines() {
        let trimmed = line.trim();
        
        // 跳過空行
        if trimmed.is_empty() {
            if !current_commit.is_empty() {
                current_commit.push('\n');
            }
            continue;
        }
        
        // 檢查是否是新 commit 的開始
        // 格式：以英文字母開頭，後接冒號，冒號後有空格或中文
        // 例如：feat: xxx、fix: xxx、custom-type: xxx
        let is_commit_start = if let Some(colon_pos) = trimmed.find(':') {
            // 冒號前面的部分
            let before_colon = &trimmed[..colon_pos];
            // 檢查：1) 不是空的，2) 只包含英文字母、數字、連字號，3) 以字母開頭
            !before_colon.is_empty() 
                && before_colon.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                && before_colon.chars().next().map_or(false, |c| c.is_ascii_alphabetic())
        } else {
            false
        };
        
        if is_commit_start {
            // 儲存前一個 commit（如果有的話）
            if !current_commit.is_empty() {
                commit_messages.push(current_commit.trim().to_string());
            }
            // 開始新的 commit
            current_commit = trimmed.to_string();
        } else {
            // 繼續累加到當前 commit
            if !current_commit.is_empty() {
                current_commit.push('\n');
                current_commit.push_str(trimmed);
            }
        }
    }
    
    // 加入最後一個 commit
    if !current_commit.is_empty() {
        commit_messages.push(current_commit.trim().to_string());
    }
    
    // 限制為 3 個
    commit_messages.truncate(3);
    
    // 確保至少有一些建議
    if !branch_names.is_empty() || !commit_messages.is_empty() {
        // 補足數量（如果不足 3 個）
        while branch_names.len() < 3 {
            let timestamp = Local::now().format("%Y%m%d").to_string();
            branch_names.push(format!("feature/update-{}", timestamp));
        }
        
        Some(GitSuggestions {
            branch_names: branch_names.into_iter().take(3).collect(),
            commit_messages: commit_messages.into_iter().take(3).collect(),
        })
    } else {
        None
    }
}

/// 備用 commit 訊息建議（當 LLM 不可用時）
fn generate_fallback_commit_suggestions(diff: &str, files: &[String]) -> Vec<String> {
    let mut suggestions = Vec::new();

    let has_new_files = diff.contains("new file mode");
    let has_deleted_files = diff.contains("deleted file mode");
    let has_code = files.iter().any(|f| {
        f.ends_with(".rs") || f.ends_with(".js") || f.ends_with(".py")
    });

    if has_new_files {
        suggestions.push("feat: 新增檔案".to_string());
    } else if has_deleted_files {
        suggestions.push("chore: 移除不需要的檔案".to_string());
    } else {
        suggestions.push("chore: 更新專案檔案".to_string());
    }

    if has_code {
        suggestions.push("fix: 修正程式錯誤".to_string());
        suggestions.push("perf: 改善程式效能".to_string());
    } else {
        suggestions.push("docs: 更新文檔內容".to_string());
        suggestions.push("chore: 日常維護更新".to_string());
    }

    suggestions.truncate(3);
    suggestions
}

/// 備用分支名稱建議（當 LLM 不可用時）
fn generate_fallback_branch_suggestions(_files: &[String]) -> Vec<String> {
    let timestamp = Local::now().format("%Y%m%d").to_string();
    
    vec![
        format!("feature/update-{}", timestamp),
        format!("fix/bug-fix-{}", timestamp),
        format!("refactor/improve-{}", timestamp),
    ]
}

/// 選擇分支
fn select_branch(current: &str, suggestions: &[String]) -> Result<Option<String>> {
    // 顯示標題
    println!("\n{}", format!("當前分支：{}", current).dimmed());
    println!("{}", "--- 建議的分支名稱 ---".cyan());
    
    let mut items = vec![format!("保持當前分支 ({})", current)];

    for (i, suggestion) in suggestions.iter().enumerate() {
        items.push(format!("{}. {}", i + 1, suggestion));
    }

    items.push("自訂分支名稱".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("請選擇")
        .items(&items)
        .default(0)
        .interact()?;

    // 保持當前分支
    if selection == 0 {
        return Ok(None);
    }

    // 自訂分支名稱
    if selection == items.len() - 1 {
        let custom_branch: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("請輸入自訂分支名稱")
            .validate_with(|input: &String| {
                if input.trim().is_empty() {
                    Err("分支名稱不能為空")
                } else if !is_valid_branch_name(input) {
                    Err("分支名稱包含無效字元")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;
        return Ok(Some(custom_branch.trim().to_string()));
    }

    // 選擇建議的分支
    let index = selection - 1; // 減去 "保持當前分支"
    if index < suggestions.len() {
        Ok(Some(suggestions[index].clone()))
    } else {
        Ok(None)
    }
}

/// 選擇 commit 訊息（包含預覽和確認循環）
fn select_commit_message(suggestions: &[String]) -> Result<String> {
    loop {
        // 顯示標題
        println!("\n{}", "--- 建議的 Commit 訊息 ---".cyan());
        
        let mut items = Vec::new();

        // 只顯示每個建議的第一行（標題），避免選單過長
        for (i, suggestion) in suggestions.iter().enumerate() {
            let first_line = suggestion.lines().next().unwrap_or(suggestion);
            items.push(format!("{}. {}", i + 1, first_line));
        }

        items.push("自訂 Commit 訊息".to_string());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("請選擇")
            .items(&items)
            .default(0)
            .interact()?;

        // 處理選擇
        let message = if selection == items.len() - 1 {
            // 自訂 commit 訊息
            let custom_message: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("請輸入自訂 Commit 訊息")
                .validate_with(|input: &String| {
                    if input.trim().is_empty() {
                        Err("Commit 訊息不能為空")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?;
            
            custom_message.trim().to_string()
        } else if selection < suggestions.len() {
            // 選擇建議的訊息
            suggestions[selection].clone()
        } else {
            continue;
        };

        // 顯示完整預覽
        println!();
        println!("{}", "📋 Commit 預覽".blue().bold());
        println!("{}", "─────────────────────────────────────".dimmed());
        println!("{}", message);
        println!("{}", "─────────────────────────────────────".dimmed());
        println!();

        // 確認或重新選擇
        let confirm_items = vec!["✓ 確認使用此訊息", "← 重新選擇"];
        let confirmed = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("請選擇")
            .items(&confirm_items)
            .default(0)
            .interact()?;

        if confirmed == 0 {
            // 確認，返回訊息
            return Ok(message);
        }
        // 否則繼續循環，重新選擇
    }
}

/// 驗證分支名稱
fn is_valid_branch_name(name: &str) -> bool {
    // Git 分支名稱規則：不能包含空格、~、^、:、?、*、[、]、\
    // 以及不能以 / 或 . 開頭
    let invalid_chars = [' ', '~', '^', ':', '?', '*', '[', ']', '\\'];
    
    if name.starts_with('/') || name.starts_with('.') {
        return false;
    }

    !name.chars().any(|c| invalid_chars.contains(&c))
}

/// 切換分支
fn switch_branch(branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(&["checkout", "-b", branch_name])
        .output()
        .context("無法執行 git checkout")?;

    if output.status.success() {
        println!("{}", format!("✓ 已切換到新分支：{}", branch_name).green());
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        println!("{}", format!("✗ 切換分支失敗：{}", error).red());
        anyhow::bail!("切換分支失敗")
    }
}

/// 執行 git commit
fn commit_changes(message: &str) -> Result<()> {
    let output = Command::new("git")
        .args(&["commit", "-m", message])
        .output()
        .context("無法執行 git commit")?;

    if output.status.success() {
        println!("{}", "✓ Commit 成功！".green());
        println!("{}", format!("  訊息：{}", message).dimmed());
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        println!("{}", format!("✗ Commit 失敗：{}", error).red());
        anyhow::bail!("Commit 失敗")
    }
}
