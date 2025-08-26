# mcp-markdown-tools

[![English](https://img.shields.io/badge/English-Click-yellow)](README.md)
[![简体中文](https://img.shields.io/badge/简体中文-点击查看-orange)](README-zh.md)

一个处理 Markdown 文档的 MCP 服务器。另外内置了一个基于 MST (Markdown structured tree) 的 Markdown 文档解析器和编译器。

## 用法

### Docker 模式

TODO...

### MCP 服务器模式

```json
{
  "mcpServers": {
    "markdown-tools": {
      "command": "mcp-markdown-tools"
    }
  }
}
```

### CLI 模式

TODO...

## 可用的工具

### check_heading

验证 Markdown 文档标题行的格式规范性和层级结构的正确性。

#### 参数

- file_path：Markdown 文档的文件路径

### generate_chapter_number

为 Markdown 文档所有的标题行(Head line)创建编号。

#### 参数

- file_path：Markdown 文档的文件路径
- ignore_h1：是否忽略一级标题（# 标题）
- use_chinese_number：是否使用中文编号（一、二、三...）
- use_arabic_number_for_sublevel：一级以下编号是否使用独立的阿拉伯数字编号。
- save_as_new_file：编辑后，是否另存为新文件，为false时将覆盖原文件。
- new_file_name：新文件名，不包括扩展名(.md)。save_as_new_file=true 时生效。

### remove_all_chapter_numbers

清除 Markdown 文档所有标题行(Head line)的编号，包括数字和中文编号。

#### 参数

- file_path：Markdown 文档的文件路径
- save_as_new_file：编辑后，是否另存为新文件，为false时将覆盖原文件。
- new_file_name：新文件名，不包括扩展名(.md)。save_as_new_file=true 时生效。

## 参考

- Markdown 语法：<https://www.markdownlang.com/cheatsheet/>
- MCP SDK for Rust: <https://github.com/modelcontextprotocol/rust-sdk/>
