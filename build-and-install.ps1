# PowerShell 腳本 - Windows 版本建置與安裝工具

# 錯誤時停止執行
$ErrorActionPreference = "Stop"

# 顏色輸出函數
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

Write-ColorOutput Blue "開始建置所有工具..."

# 建置所有工具
cargo build --release --workspace

if ($LASTEXITCODE -ne 0) {
    Write-ColorOutput Red "✗ 建置失敗"
    exit 1
}

Write-ColorOutput Green "✓ 建置完成"

# 建立 ~/bin/ 目錄（如果不存在）
$BinDir = Join-Path $env:USERPROFILE "bin"
if (-not (Test-Path $BinDir)) {
    Write-ColorOutput Yellow "建立目錄: $BinDir"
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

# 取得建置目錄
$ReleaseDir = "target\release"

# 檢查建置目錄是否存在
if (-not (Test-Path $ReleaseDir)) {
    Write-ColorOutput Yellow "⚠️  建置目錄不存在，請先執行建置"
    exit 1
}

Write-ColorOutput Blue "複製執行檔至 $BinDir"

# 取得專案根目錄的絕對路徑
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

# 複製所有 .exe 執行檔
$installedCount = 0
Get-ChildItem -Path $ReleaseDir -Filter "*.exe" | ForEach-Object {
    $binaryName = $_.Name
    $sourcePath = $_.FullName
    $destPath = Join-Path $BinDir $binaryName
    
    Write-Output "  複製: $binaryName"
    Copy-Item -Path $sourcePath -Destination $destPath -Force
    $installedCount++
}

if ($installedCount -eq 0) {
    Write-ColorOutput Yellow "⚠️  未找到可安裝的執行檔"
    exit 1
}

Write-ColorOutput Green "✓ 所有工具已安裝至 $BinDir"
Write-Output ""
Write-ColorOutput Yellow "提示：請確保 $BinDir 已加入 PATH 環境變數"
Write-ColorOutput Yellow "如果尚未加入，可以執行以下命令（需要管理員權限）："
Write-Output '  $env:Path += ";$env:USERPROFILE\bin"'
Write-Output '  [Environment]::SetEnvironmentVariable("Path", $env:Path, [System.EnvironmentVariableTarget]::User)'
Write-Output ""
Write-ColorOutput Yellow "或手動新增到系統環境變數："
Write-Output "  1. 開啟「系統內容」→「進階系統設定」→「環境變數」"
Write-Output "  2. 在「使用者變數」中編輯 Path"
Write-Output "  3. 新增: $BinDir"
