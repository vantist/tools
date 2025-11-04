#!/usr/bin/env bash

set -e

# 顏色輸出
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}開始建置所有工具...${NC}"

# 建置所有工具
cargo build --release --workspace

echo -e "${GREEN}✓ 建置完成${NC}"

# 建立 ~/bin/ 目錄（如果不存在）
BIN_DIR="$HOME/bin"
if [ ! -d "$BIN_DIR" ]; then
    echo -e "${YELLOW}建立目錄: $BIN_DIR${NC}"
    mkdir -p "$BIN_DIR"
fi

# 取得所有建置的執行檔
RELEASE_DIR="target/release"

# 檢查建置目錄是否存在
if [ ! -d "$RELEASE_DIR" ]; then
    echo -e "${YELLOW}⚠️  建置目錄不存在，請先執行建置${NC}"
    exit 1
fi

echo -e "${BLUE}將執行檔連結至 $BIN_DIR/${NC}"

# 取得專案根目錄的絕對路徑
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 連結所有執行檔
installed_count=0
for binary in "$RELEASE_DIR"/*; do
    # 只處理可執行檔，且排除 .d, .rlib, .so 等非可執行檔
    if [ -f "$binary" ] && [ -x "$binary" ] && [[ ! "$binary" =~ \.(d|rlib|so)$ ]]; then
        binary_name=$(basename "$binary")
        binary_path="$PROJECT_ROOT/$binary"
        echo -e "  連結: ${GREEN}$binary_name${NC}"
        ln -sf "$binary_path" "$BIN_DIR/$binary_name"
        installed_count=$((installed_count + 1))
    fi
done

if [ $installed_count -eq 0 ]; then
    echo -e "${YELLOW}⚠️  未找到可安裝的執行檔${NC}"
    exit 1
fi

echo -e "${GREEN}✓ 所有工具已安裝至 $BIN_DIR/${NC}"
echo ""
echo -e "${YELLOW}提示：請確保 $BIN_DIR 已加入 PATH 環境變數${NC}"
echo -e "${YELLOW}如果尚未加入，請在 ~/.bashrc 或 ~/.zshrc 中加入：${NC}"
echo -e "  export PATH=\"\$HOME/bin:\$PATH\""
