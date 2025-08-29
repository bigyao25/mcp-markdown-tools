# mcp-markdown-tools

[![English](https://img.shields.io/badge/English-Click-yellow)](README.md)
[![简体中文](https://img.shields.io/badge/简体中文-点击查看-orange)](README-zh.md)

一个高效处理 Markdown 文档的 MCP 服务器。

## 特性

- 检查 Markdown 文档的标题行符号`#`和编号的有效性
- 创建和清除 Markdown 文档的标题编号，支持阿拉伯数字和中文数字
- 高效和低成本：全文批量处理，避免面对大文档时 AI 逐次分块处理的默认行为，文件越大效果越明显，加快处理速度的同时节省 token 消耗
- 超轻量：基于 Rust 开发的二进制程序，CPU 占用和驻留内存可忽略不计
- 内置了一个基于 MST (Markdown structured tree) 的 Markdown 文档解析器和编译器

## 用法

### 安装

#### Docker 模式

将由 Docker 托管的 MCP 服务器加入到你的 LLM 助手应用中，配置文件如下：

  ```json
  {
    "mcpServers": {
      "markdown-tools": {
        "command": "docker",
        "args": [
          "run",
          "--rm",
          "-i",
          "--init",
          "-v",
          "/Users:/Users",
          "bigyao25/mcp-markdown-tools"
        ]
      }
    }
  }
  ```

#### 二进制程序模式

1. 从 [Release](https://github.com/bigyao25/mcp-markdown-tools/releases) 下载适合你运行环境的最新的稳定版二进制文件

2. 在你的 LLM 助手应用中加入本 MCP 服务器，配置文件如下：

```json
{
  "mcpServers": {
    "markdown-tools": {
      "command": "/real/path/to/mcp-markdown-tools"
    }
  }
}
```

### 对话示例

- 检查 `/home/docs/lorem.md` 标题级别的逻辑
- 清除 `/home/docs/lorem.md` 所有标题中的编号
- 将 `/home/docs/lorem.md` 二级及其以下的标题添加编号，另存为 `lorem-numed.md`
- 请帮我把 `/home/docs/doc1-cn.md` 的标题添加中文编号，首行不加，另存为：`doc1-cn-numed.md`

## 可用的工具

### check_heading

验证 Markdown 文档标题行的格式规范性和层级结构的正确性。

#### 参数

- full_file_path：Markdown 文档的文件路径

### generate_chapter_number

为 Markdown 文档所有的标题行(Head line)创建编号。

#### 参数

- full_file_path：Markdown 文档的文件路径
- ignore_h1：是否忽略一级标题（# 标题）
- use_chinese_number：是否使用中文编号（一、二、三...）
- use_arabic_number_for_sublevel：一级以下编号是否使用独立的阿拉伯数字编号。
- save_as_new_file：编辑后，是否另存为新文件，为false时将覆盖原文件。
- new_full_file_path：新文件名。save_as_new_file=true 时生效。

### remove_all_chapter_numbers

清除 Markdown 文档所有标题行(Head line)的编号，包括数字和中文编号。

#### 参数

- full_file_path：Markdown 文档的文件路径
- save_as_new_file：编辑后，是否另存为新文件，为false时将覆盖原文件。
- new_full_file_path：新文件名。save_as_new_file=true 时生效。

## TODO

- 内嵌远程图片本地化
- 文件比较

## 参考

- Markdown 语法：<https://www.markdownlang.com/basic/headings.html>
- MCP SDK for Rust: <https://github.com/modelcontextprotocol/rust-sdk/>
