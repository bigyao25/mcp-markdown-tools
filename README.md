# mcp-markdown-tools

[![English](https://img.shields.io/badge/English-Click-yellow)](README.md)
[![简体中文](https://img.shields.io/badge/简体中文-点击查看-orange)](README-zh.md)

An efficient MCP server for processing Markdown documents.

## Features

- Check the validity of Markdown document heading line symbols `#` and numbering
- Create and remove heading numbering in Markdown documents, supporting both Arabic numerals and Chinese numerals
- Efficient and cost-effective: Batch processing of entire documents to avoid AI's default behavior of processing large documents in chunks, with more significant effects on larger files, speeding up processing while saving token consumption
- Ultra-lightweight: Binary program developed in Rust with negligible CPU usage and resident memory
- Built-in Markdown document parser and compiler based on MST (Markdown structured tree)

## Usage

### Installation

#### Docker Mode

Add the Docker-hosted MCP server to your LLM assistant application with the following configuration:

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

#### Binary Program Mode

1. Download the latest stable binary file suitable for your runtime environment from [Release](https://github.com/bigyao25/mcp-markdown-tools/releases)

2. Add this MCP server to your LLM assistant application with the following configuration:

```json
{
  "mcpServers": {
    "markdown-tools": {
      "command": "/real/path/to/mcp-markdown-tools"
    }
  }
}
```

### Conversation Examples

- Check the heading level logic of `/home/docs/lorem.md`
- Remove all numbering from headings in `/home/docs/lorem.md`
- Add numbering to level 2 and below headings in `/home/docs/lorem.md`, save as `lorem-numed.md`
- Please help me add Chinese numbering to the headings in `/home/docs/doc1-cn.md`, skip the first line, save as: `doc1-cn-numed.md`

## Available Tools

### check_heading

Validates the format compliance and hierarchical structure correctness of Markdown document heading lines.

#### Parameters

- full_file_path: File path of the Markdown document

### generate_chapter_number

Creates numbering for all heading lines in a Markdown document.

#### Parameters

- full_file_path: File path of the Markdown document
- ignore_h1: Whether to ignore level 1 headings (# headings)
- use_chinese_number: Whether to use Chinese numbering (一、二、三...)
- use_arabic_number_for_sublevel: Whether sub-level numbering below level 1 uses independent Arabic numbering
- save_as_new_file: Whether to save as a new file after editing; when false, the original file will be overwritten
- new_full_file_path: New file name. Takes effect when save_as_new_file=true

### remove_all_chapter_numbers

Removes all numbering from heading lines in a Markdown document, including both numeric and Chinese numbering.

#### Parameters

- full_file_path: File path of the Markdown document
- save_as_new_file: Whether to save as a new file after editing; when false, the original file will be overwritten
- new_full_file_path: New file name. Takes effect when save_as_new_file=true

## TODO

- Localize embedded remote images
- File comparison

## References

- Markdown Syntax: <https://www.markdownlang.com/basic/headings.html>
- MCP SDK for Rust: <https://github.com/modelcontextprotocol/rust-sdk/>
