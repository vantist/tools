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
echo -e "${BLUE}將執行檔連結至 $BIN_DIR/${NC}"

# 連結所有執行檔
for binary in "$RELEASE_DIR"/*; do
    # 只處理可執行檔，且排除 .d 和其他非執行檔
    if [ -f "$binary" ] && [ -x "$binary" ] && [[ ! "$binary" =~ \. ]]; then
        binary_name=$(basename "$binary")
        echo -e "  連結: ${GREEN}$binary_name${NC}"
        ln -f "$binary" "$BIN_DIR/$binary_name"
    fi
done

echo -e "${GREEN}✓ 所有工具已安裝至 $BIN_DIR/${NC}"
echo ""
echo -e "${YELLOW}提示：請確保 $BIN_DIR 已加入 PATH 環境變數${NC}"
echo -e "如果尚未加入，請在 ~/.bashrc 或 ~/.zshrc 中加入：${NC}"
echo -e "  export PATH=\"\$HOME/bin:\$PATH\""
