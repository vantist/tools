# Tools

這是一個工具集，透過 AI 建立簡化工作流程的工具。

## 專案結構

本專案使用 Rust 作為語言基底，每個工具都有獨立的資料夾。

```
tools/
├── Cargo.toml          # Workspace 配置
├── tools/              # 各個工具的資料夾
│   ├── example-tool/   # 範例工具
│   └── ...             # 其他工具
└── README.md
```

## 開發指南

### 前置需求

- Rust 1.70 或更新版本
- Cargo

### 建立新工具

```bash
cargo new --bin tools/your-tool-name
```

### 編譯所有工具

```bash
cargo build
```

### 執行測試

```bash
cargo test
```

## 授權

MIT License - 詳見 [LICENSE](LICENSE) 檔案