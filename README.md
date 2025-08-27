# mcp-markdown-tools

[![English](https://img.shields.io/badge/English-Click-yellow)](README.md)
[![简体中文](https://img.shields.io/badge/简体中文-点击查看-orange)](README-zh.md)

An MCP server for processing Markdown documents. It also includes a built-in Markdown document parser and compiler based on MST (Markdown structured tree).

## Usage

### Docker Mode

TODO...

### MCP Server Mode

```json
{
  "mcpServers": {
    "markdown-tools": {
      "command": "mcp-markdown-tools"
    }
  }
}
```

### CLI Mode

TODO...

## Available Tools

### check_heading

Validates the format compliance and hierarchical structure correctness of Markdown document heading lines.

#### Parameters

- file_path: File path of the Markdown document

### generate_chapter_number

Creates numbering for all heading lines in a Markdown document.

#### Parameters

- file_path: File path of the Markdown document
- ignore_h1: Whether to ignore level 1 headings (# headings)
- use_chinese_number: Whether to use Chinese numbering (一、二、三...)
- use_arabic_number_for_sublevel: Whether to use independent Arabic numbering for sub-levels below level 1
- save_as_new_file: Whether to save as a new file after editing; when false, the original file will be overwritten
- new_file_name: New file name, excluding the extension (.md). Takes effect when save_as_new_file=true

### remove_all_chapter_numbers

Removes all numbering from heading lines in a Markdown document, including both numeric and Chinese numbering.

#### Parameters

- file_path: File path of the Markdown document
- save_as_new_file: Whether to save as a new file after editing; when false, the original file will be overwritten
- new_file_name: New file name, excluding the extension (.md). Takes effect when save_as_new_file=true

## Release

1. Append new version info into CHANGELOG.md

  ```markdown
  ## [v0.1.0] - 2025-01-01

  ### Added

  - 新功能A
  - 新功能B

  ### Fixed

  - 修复问题C
  ```

2. Create and push new tag

```bash
git tag v0.1.0
git push origin v0.1.0
```

## References

- Markdown Syntax: <https://www.markdownlang.com/cheatsheet/>
- MCP SDK for Rust: <https://github.com/modelcontextprotocol/rust-sdk/>
