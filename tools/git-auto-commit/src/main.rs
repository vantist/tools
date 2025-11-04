use anyhow::{Context, Result};
use chrono::Local;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use git2::{Repository, StatusOptions};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<()> {
    println!("\n{}\n", "🚀 Git 自動 Commit 工具".cyan().bold());

    // 檢查是否在 git repository 中
    let repo = Repository::open(".").context("✗ 錯誤：當前目錄不是 Git repository")?;

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

    // 生成建議
    let branch_suggestions = generate_branch_suggestions(&staged_files);
    let commit_suggestions = generate_commit_suggestions(&diff_content, &staged_files);

    // 詢問是否要切換分支
    let branch_choice = select_branch(&current_branch, &branch_suggestions)?;

    // 處理分支切換
    if let Some(new_branch) = branch_choice {
        switch_branch(&new_branch)?;
    }

    println!();

    // 詢問 commit 訊息
    let commit_message = select_commit_message(&commit_suggestions)?;

    println!();

    // 確認並執行 commit
    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("確認要 commit？\n  訊息：{}", commit_message))
        .default(true)
        .interact()?;

    if confirmed {
        commit_changes(&commit_message)?;
    } else {
        println!("{}", "✗ 已取消 commit".yellow());
    }

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

/// 取得 staged 的 diff 內容
fn get_staged_diff(_repo: &Repository) -> Result<String> {
    let output = Command::new("git")
        .args(&["diff", "--staged"])
        .output()
        .context("無法執行 git diff")?;

    if !output.status.success() {
        anyhow::bail!("git diff 執行失敗");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
    vec!["--yolo".to_string()]
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            command: default_command(),
            prompt_flag: default_prompt_flag(),
            model_flag: default_model_flag(),
            model: default_model(),
            extra_args: default_extra_args(),
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

/// 使用 Gemini CLI 生成建議
fn call_llm_cli(prompt: &str) -> Result<String> {
    let config = load_llm_config();
    
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

/// 生成 commit 訊息建議（使用 LLM）
fn generate_commit_suggestions(diff: &str, files: &[String]) -> Vec<String> {
    println!("{}", "🤖 正在使用 LLM 生成 commit 訊息建議...".dimmed());
    
    // 限制 diff 長度以避免超過 API 限制
    let diff_preview = if diff.len() > 3000 {
        &diff[..3000]
    } else {
        diff
    };

    let files_list = files.join(", ");
    let prompt = format!(
        r#"你是一個 Git commit 訊息專家。請根據以下 git diff 內容和檔案列表，生成 3 個簡潔的繁體中文 commit 訊息建議。

檔案列表：
{}

Git diff：
```
{}
```

要求：
1. 每個建議一行
2. 使用繁體中文
3. 格式：「類型：簡短描述」（例如：「修復：修正登入錯誤」、「新增：添加使用者管理功能」）
4. 常用類型包括：新增、修復、更新、重構、文檔、測試、優化、配置、刪除、清理
5. 描述要簡潔明瞭，不超過 50 字
6. 只回傳 3 個建議，每行一個，不要有其他說明文字
7. 不要使用 markdown 格式，不要編號"#,
        files_list, diff_preview
    );

    match call_llm_cli(&prompt) {
        Ok(response) => {
            let suggestions: Vec<String> = response
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .take(3)
                .collect();

            if suggestions.len() == 3 {
                return suggestions;
            }
        }
        Err(e) => {
            println!("{}", format!("⚠️  LLM 生成失敗：{}", e).yellow());
            println!("{}", "使用備用建議...".dimmed());
        }
    }

    // 備用建議（如果 LLM 失敗）
    generate_fallback_commit_suggestions(diff, files)
}

/// 生成分支名稱建議（使用 LLM）
fn generate_branch_suggestions(files: &[String]) -> Vec<String> {
    println!("{}", "🤖 正在使用 LLM 生成分支名稱建議...".dimmed());
    
    let files_list = files.join(", ");
    let timestamp = Local::now().format("%Y%m%d").to_string();
    
    let prompt = format!(
        r#"你是一個 Git 分支命名專家。請根據以下檔案列表，生成 3 個符合規範的分支名稱建議。

檔案列表：
{}

要求：
1. 每個建議一行
2. 格式：「類型/描述-{}」（例如：「feature/add-user-auth-{}」、「fix/login-bug-{}」）
3. 常用類型：feature（新功能）、fix（修復）、refactor（重構）、docs（文檔）、test（測試）、chore（維護）、config（配置）
4. 描述使用英文小寫，單字之間用連字號 - 連接
5. 描述要簡潔，不超過 30 字元
6. 只回傳 3 個建議，每行一個，不要有其他說明文字
7. 不要使用 markdown 格式，不要編號"#,
        files_list, timestamp, timestamp, timestamp
    );

    match call_llm_cli(&prompt) {
        Ok(response) => {
            let suggestions: Vec<String> = response
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .take(3)
                .collect();

            if suggestions.len() == 3 {
                return suggestions;
            }
        }
        Err(e) => {
            println!("{}", format!("⚠️  LLM 生成失敗：{}", e).yellow());
            println!("{}", "使用備用建議...".dimmed());
        }
    }

    // 備用建議（如果 LLM 失敗）
    generate_fallback_branch_suggestions(files)
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
        suggestions.push("新增：添加新檔案".to_string());
    } else if has_deleted_files {
        suggestions.push("刪除：移除不需要的檔案".to_string());
    } else {
        suggestions.push("更新：更新專案檔案".to_string());
    }

    if has_code {
        suggestions.push("修復：修正程式錯誤".to_string());
        suggestions.push("優化：改善程式效能".to_string());
    } else {
        suggestions.push("文檔：更新文檔內容".to_string());
        suggestions.push("維護：日常維護更新".to_string());
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
    let mut items = vec![format!("保持當前分支 ({})", current)];
    items.push("--- 建議的分支名稱 ---".to_string());

    for (i, suggestion) in suggestions.iter().enumerate() {
        items.push(format!("{}. {}", i + 1, suggestion));
    }

    items.push("──────────────".to_string());
    items.push("自訂分支名稱".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("選擇分支")
        .items(&items)
        .default(0)
        .interact()?;

    // 保持當前分支
    if selection == 0 {
        return Ok(None);
    }

    // 分隔線不應該被選擇，但為了安全起見處理
    if selection == 1 || selection == items.len() - 2 {
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
    let index = selection - 2; // 減去 "保持當前分支" 和分隔線
    if index < suggestions.len() {
        Ok(Some(suggestions[index].clone()))
    } else {
        Ok(None)
    }
}

/// 選擇 commit 訊息
fn select_commit_message(suggestions: &[String]) -> Result<String> {
    let mut items = vec!["--- 建議的 Commit 訊息 ---".to_string()];

    for (i, suggestion) in suggestions.iter().enumerate() {
        items.push(format!("{}. {}", i + 1, suggestion));
    }

    items.push("──────────────".to_string());
    items.push("自訂 Commit 訊息".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("選擇 Commit 訊息")
        .items(&items)
        .default(1)
        .interact()?;

    // 分隔線
    if selection == 0 || selection == items.len() - 2 {
        return select_commit_message(suggestions);
    }

    // 自訂 commit 訊息
    if selection == items.len() - 1 {
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
        return Ok(custom_message.trim().to_string());
    }

    // 選擇建議的訊息
    let index = selection - 1; // 減去分隔線
    if index < suggestions.len() {
        Ok(suggestions[index].clone())
    } else {
        select_commit_message(suggestions)
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
