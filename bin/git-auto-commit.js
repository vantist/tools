#!/usr/bin/env node

const { spawnSync } = require('child_process');
const inquirer = require('inquirer');
const chalk = require('chalk');

/**
 * 執行 git 指令並回傳輸出
 * 使用 spawnSync 來安全地傳遞參數，避免指令注入
 */
function executeGitCommand(command, args) {
  try {
    const result = spawnSync(command, args, { encoding: 'utf-8' });
    if (result.error) {
      throw result.error;
    }
    if (result.status !== 0) {
      throw new Error(result.stderr || 'Command failed');
    }
    return result.stdout;
  } catch (error) {
    throw error;
  }
}

/**
 * 取得當前 staged 的差異
 */
function getStagedDiff() {
  try {
    const diff = executeGitCommand('git', ['diff', '--staged']);
    if (!diff || diff.trim() === '') {
      console.log(chalk.yellow('⚠️  沒有 staged 的檔案變更，請先使用 git add 加入檔案'));
      process.exit(1);
    }
    return diff;
  } catch (error) {
    console.log(chalk.red(`✗ 無法取得 staged 差異：${error.message}`));
    process.exit(1);
  }
}

/**
 * 取得當前分支名稱
 */
function getCurrentBranch() {
  try {
    const branch = executeGitCommand('git', ['branch', '--show-current']);
    return branch ? branch.trim() : 'main';
  } catch (error) {
    console.log(chalk.yellow(`⚠️  無法取得當前分支，使用預設值 'main'`));
    return 'main';
  }
}

/**
 * 取得 staged 的檔案列表
 */
function getStagedFiles() {
  try {
    const files = executeGitCommand('git', ['diff', '--staged', '--name-only']);
    return files ? files.trim().split('\n').filter(f => f) : [];
  } catch (error) {
    console.log(chalk.red(`✗ 無法取得檔案列表：${error.message}`));
    return [];
  }
}

/**
 * 分析 diff 內容，生成 commit 訊息建議
 */
function generateCommitSuggestions(diff, files) {
  const suggestions = [];
  
  // 分析檔案類型和變更
  const hasNewFiles = diff.includes('new file mode');
  const hasDeletedFiles = diff.includes('deleted file mode');
  const hasModifiedFiles = diff.includes('diff --git') && !diff.includes('new file mode') && !diff.includes('deleted file mode');
  
  // 分析檔案類型
  const fileTypes = {
    docs: files.some(f => f.match(/\.(md|txt|doc)$/i)),
    config: files.some(f => f.match(/\.(json|yaml|yml|toml|ini|conf)$/i)),
    scripts: files.some(f => f.match(/\.(sh|bash|bat|cmd)$/i)),
    code: files.some(f => f.match(/\.(js|ts|py|java|cpp|c|go|rb|php)$/i)),
    styles: files.some(f => f.match(/\.(css|scss|sass|less)$/i)),
    tests: files.some(f => f.match(/test|spec/i)),
  };
  
  // 根據變更類型生成建議
  if (hasNewFiles) {
    suggestions.push('新增：' + (files.length > 1 ? '添加新檔案' : `添加 ${files[0]}`));
    if (fileTypes.docs) {
      suggestions.push('文檔：新增專案文檔');
    } else if (fileTypes.config) {
      suggestions.push('配置：新增設定檔');
    } else if (fileTypes.code) {
      suggestions.push('功能：新增功能模組');
    }
  } else if (hasDeletedFiles) {
    suggestions.push('刪除：' + (files.length > 1 ? '移除不需要的檔案' : `移除 ${files[0]}`));
    suggestions.push('清理：清理過時的程式碼');
    suggestions.push('重構：移除冗餘檔案');
  } else if (hasModifiedFiles) {
    if (fileTypes.docs) {
      suggestions.push('文檔：更新專案說明文件');
      suggestions.push('文檔：修正文檔內容');
    } else if (fileTypes.config) {
      suggestions.push('配置：調整專案設定');
      suggestions.push('配置：更新設定檔');
    } else if (fileTypes.tests) {
      suggestions.push('測試：更新測試案例');
      suggestions.push('測試：修正測試程式');
    } else if (fileTypes.code) {
      suggestions.push('修復：修正程式錯誤');
      suggestions.push('優化：改善程式效能');
      suggestions.push('重構：重構程式碼結構');
    } else if (fileTypes.styles) {
      suggestions.push('樣式：調整介面樣式');
      suggestions.push('UI：更新使用者介面');
    }
  }
  
  // 通用建議（如果上面沒有產生足夠的建議）
  if (suggestions.length < 3) {
    const genericSuggestions = [
      '更新：更新專案檔案',
      '改進：改善程式碼品質',
      '維護：日常維護更新',
      '調整：調整檔案內容',
      '修改：修改專案檔案',
    ];
    
    for (const suggestion of genericSuggestions) {
      if (suggestions.length >= 3) break;
      if (!suggestions.includes(suggestion)) {
        suggestions.push(suggestion);
      }
    }
  }
  
  return suggestions.slice(0, 3);
}

/**
 * 生成分支名稱建議
 */
function generateBranchSuggestions(files) {
  const suggestions = [];
  const timestamp = new Date().toISOString().slice(0, 10).replace(/-/g, '');
  
  // 分析檔案類型
  const hasFeature = files.some(f => f.includes('feature') || f.includes('新增') || f.includes('add'));
  const hasFix = files.some(f => f.includes('fix') || f.includes('修復') || f.includes('bug'));
  const hasDocs = files.some(f => f.match(/\.(md|txt|doc)$/i));
  const hasConfig = files.some(f => f.match(/\.(json|yaml|yml|toml|ini|conf)$/i));
  const hasTest = files.some(f => f.match(/test|spec/i));
  
  if (hasFeature) {
    suggestions.push(`feature/new-feature-${timestamp}`);
  }
  if (hasFix) {
    suggestions.push(`fix/bug-fix-${timestamp}`);
  }
  if (hasDocs) {
    suggestions.push(`docs/update-docs-${timestamp}`);
  }
  if (hasConfig) {
    suggestions.push(`config/update-config-${timestamp}`);
  }
  if (hasTest) {
    suggestions.push(`test/update-tests-${timestamp}`);
  }
  
  // 通用建議
  const genericSuggestions = [
    `feature/update-${timestamp}`,
    `refactor/improve-code-${timestamp}`,
    `chore/maintenance-${timestamp}`,
  ];
  
  for (const suggestion of genericSuggestions) {
    if (suggestions.length >= 3) break;
    if (!suggestions.includes(suggestion)) {
      suggestions.push(suggestion);
    }
  }
  
  return suggestions.slice(0, 3);
}

/**
 * 驗證分支名稱格式
 */
function isValidBranchName(branchName) {
  // Git 分支名稱規則：不能包含空格、~、^、:、?、*、[、]、\、以及不能以 / 或 . 開頭
  const invalidCharsRegex = /[\s~^:?*[\]\\]/;
  const invalidStartRegex = /^[/.]/;
  
  return !invalidCharsRegex.test(branchName) && !invalidStartRegex.test(branchName) && branchName.length > 0;
}

/**
 * 切換到新分支
 */
function switchBranch(branchName) {
  try {
    // 驗證分支名稱
    if (!isValidBranchName(branchName)) {
      console.log(chalk.red(`✗ 無效的分支名稱：${branchName}`));
      return false;
    }
    
    // 使用安全的方式執行指令，避免指令注入
    executeGitCommand('git', ['checkout', '-b', branchName]);
    console.log(chalk.green(`✓ 已切換到新分支：${branchName}`));
    return true;
  } catch (error) {
    console.log(chalk.red(`✗ 切換分支失敗：${error.message}`));
    return false;
  }
}

/**
 * 執行 git commit
 */
function commitChanges(message) {
  try {
    // 使用安全的方式執行指令，避免指令注入
    executeGitCommand('git', ['commit', '-m', message]);
    console.log(chalk.green(`✓ Commit 成功！`));
    console.log(chalk.gray(`  訊息：${message}`));
    return true;
  } catch (error) {
    console.log(chalk.red(`✗ Commit 失敗：${error.message}`));
    return false;
  }
}

/**
 * 主程式
 */
async function main() {
  console.log(chalk.cyan.bold('\n🚀 Git 自動 Commit 工具\n'));
  
  // 檢查是否在 git repository 中
  try {
    executeGitCommand('git', ['rev-parse', '--git-dir']);
  } catch (error) {
    console.log(chalk.red('✗ 錯誤：當前目錄不是 Git repository'));
    process.exit(1);
  }
  
  // 取得當前分支
  const currentBranch = getCurrentBranch();
  console.log(chalk.gray(`當前分支：${currentBranch}\n`));
  
  // 取得 staged diff 和檔案列表
  const diff = getStagedDiff();
  const files = getStagedFiles();
  
  console.log(chalk.blue('📝 Staged 檔案：'));
  files.forEach(file => console.log(chalk.gray(`  - ${file}`)));
  console.log();
  
  // 生成建議
  const commitSuggestions = generateCommitSuggestions(diff, files);
  const branchSuggestions = generateBranchSuggestions(files);
  
  // 詢問是否要切換分支
  const branchChoices = [
    { name: `保持當前分支 (${currentBranch})`, value: null },
    new inquirer.Separator('--- 建議的分支名稱 ---'),
    ...branchSuggestions.map((branch, idx) => ({ name: `${idx + 1}. ${branch}`, value: branch })),
    new inquirer.Separator(),
    { name: '自訂分支名稱', value: 'custom' },
  ];
  
  const { selectedBranch } = await inquirer.prompt([
    {
      type: 'list',
      name: 'selectedBranch',
      message: '選擇分支：',
      choices: branchChoices,
    },
  ]);
  
  // 處理分支切換
  if (selectedBranch === 'custom') {
    const { customBranch } = await inquirer.prompt([
      {
        type: 'input',
        name: 'customBranch',
        message: '請輸入自訂分支名稱：',
        validate: (input) => {
          if (!input || input.trim() === '') {
            return '分支名稱不能為空';
          }
          return true;
        },
      },
    ]);
    switchBranch(customBranch.trim());
  } else if (selectedBranch) {
    switchBranch(selectedBranch);
  }
  
  console.log();
  
  // 詢問 commit 訊息
  const commitChoices = [
    new inquirer.Separator('--- 建議的 Commit 訊息 ---'),
    ...commitSuggestions.map((msg, idx) => ({ name: `${idx + 1}. ${msg}`, value: msg })),
    new inquirer.Separator(),
    { name: '自訂 Commit 訊息', value: 'custom' },
  ];
  
  const { selectedCommit } = await inquirer.prompt([
    {
      type: 'list',
      name: 'selectedCommit',
      message: '選擇 Commit 訊息：',
      choices: commitChoices,
    },
  ]);
  
  // 處理 commit
  let commitMessage = selectedCommit;
  if (selectedCommit === 'custom') {
    const { customCommit } = await inquirer.prompt([
      {
        type: 'input',
        name: 'customCommit',
        message: '請輸入自訂 Commit 訊息：',
        validate: (input) => {
          if (!input || input.trim() === '') {
            return 'Commit 訊息不能為空';
          }
          return true;
        },
      },
    ]);
    commitMessage = customCommit.trim();
  }
  
  console.log();
  
  // 確認後執行 commit
  const { confirm } = await inquirer.prompt([
    {
      type: 'confirm',
      name: 'confirm',
      message: `確認要 commit？\n  訊息：${commitMessage}`,
      default: true,
    },
  ]);
  
  if (confirm) {
    commitChanges(commitMessage);
  } else {
    console.log(chalk.yellow('✗ 已取消 commit'));
  }
  
  console.log();
}

// 執行主程式
main().catch(error => {
  console.error(chalk.red(`錯誤：${error.message}`));
  process.exit(1);
});
