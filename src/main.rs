use anyhow::{Context, Result};
use chrono::Local;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use git2::{Repository, StatusOptions};
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

/// 生成 commit 訊息建議
fn generate_commit_suggestions(diff: &str, files: &[String]) -> Vec<String> {
    let mut suggestions = Vec::new();

    // 分析檔案類型和變更
    let has_new_files = diff.contains("new file mode");
    let has_deleted_files = diff.contains("deleted file mode");
    let has_modified_files = diff.contains("diff --git") && !has_new_files && !has_deleted_files;

    // 分析檔案類型
    let has_docs = files
        .iter()
        .any(|f| f.ends_with(".md") || f.ends_with(".txt") || f.ends_with(".doc"));
    let has_config = files.iter().any(|f| {
        f.ends_with(".json")
            || f.ends_with(".yaml")
            || f.ends_with(".yml")
            || f.ends_with(".toml")
            || f.ends_with(".ini")
    });
    let has_code = files.iter().any(|f| {
        f.ends_with(".rs")
            || f.ends_with(".js")
            || f.ends_with(".ts")
            || f.ends_with(".py")
            || f.ends_with(".java")
            || f.ends_with(".go")
    });
    let has_tests = files.iter().any(|f| f.contains("test") || f.contains("spec"));

    // 根據變更類型生成建議
    if has_new_files {
        if files.len() == 1 {
            suggestions.push(format!("新增：添加 {}", files[0]));
        } else {
            suggestions.push("新增：添加新檔案".to_string());
        }
        if has_docs {
            suggestions.push("文檔：新增專案文檔".to_string());
        } else if has_config {
            suggestions.push("配置：新增設定檔".to_string());
        } else if has_code {
            suggestions.push("功能：新增功能模組".to_string());
        }
    } else if has_deleted_files {
        if files.len() == 1 {
            suggestions.push(format!("刪除：移除 {}", files[0]));
        } else {
            suggestions.push("刪除：移除不需要的檔案".to_string());
        }
        suggestions.push("清理：清理過時的程式碼".to_string());
        suggestions.push("重構：移除冗餘檔案".to_string());
    } else if has_modified_files {
        if has_docs {
            suggestions.push("文檔：更新專案說明文件".to_string());
            suggestions.push("文檔：修正文檔內容".to_string());
        } else if has_config {
            suggestions.push("配置：調整專案設定".to_string());
            suggestions.push("配置：更新設定檔".to_string());
        } else if has_tests {
            suggestions.push("測試：更新測試案例".to_string());
            suggestions.push("測試：修正測試程式".to_string());
        } else if has_code {
            suggestions.push("修復：修正程式錯誤".to_string());
            suggestions.push("優化：改善程式效能".to_string());
            suggestions.push("重構：重構程式碼結構".to_string());
        }
    }

    // 通用建議
    let generic = vec![
        "更新：更新專案檔案",
        "改進：改善程式碼品質",
        "維護：日常維護更新",
        "調整：調整檔案內容",
        "修改：修改專案檔案",
    ];

    for suggestion in generic {
        if suggestions.len() >= 3 {
            break;
        }
        let s = suggestion.to_string();
        if !suggestions.contains(&s) {
            suggestions.push(s);
        }
    }

    suggestions.truncate(3);
    suggestions
}

/// 生成分支名稱建議
fn generate_branch_suggestions(files: &[String]) -> Vec<String> {
    let mut suggestions = Vec::new();
    let timestamp = Local::now().format("%Y%m%d").to_string();

    // 分析檔案類型
    let has_feature = files.iter().any(|f| f.contains("feature") || f.contains("add"));
    let has_fix = files.iter().any(|f| f.contains("fix") || f.contains("bug"));
    let has_docs = files
        .iter()
        .any(|f| f.ends_with(".md") || f.ends_with(".txt"));
    let has_config = files.iter().any(|f| {
        f.ends_with(".json")
            || f.ends_with(".yaml")
            || f.ends_with(".yml")
            || f.ends_with(".toml")
    });
    let has_test = files.iter().any(|f| f.contains("test") || f.contains("spec"));

    if has_feature {
        suggestions.push(format!("feature/new-feature-{}", timestamp));
    }
    if has_fix {
        suggestions.push(format!("fix/bug-fix-{}", timestamp));
    }
    if has_docs {
        suggestions.push(format!("docs/update-docs-{}", timestamp));
    }
    if has_config {
        suggestions.push(format!("config/update-config-{}", timestamp));
    }
    if has_test {
        suggestions.push(format!("test/update-tests-{}", timestamp));
    }

    // 通用建議
    let generic = vec![
        format!("feature/update-{}", timestamp),
        format!("refactor/improve-code-{}", timestamp),
        format!("chore/maintenance-{}", timestamp),
    ];

    for suggestion in generic {
        if suggestions.len() >= 3 {
            break;
        }
        if !suggestions.contains(&suggestion) {
            suggestions.push(suggestion);
        }
    }

    suggestions.truncate(3);
    suggestions
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
