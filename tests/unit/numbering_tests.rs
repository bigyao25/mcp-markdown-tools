//! 编号功能单元测试
//!
//! 测试章节编号生成和移除的核心逻辑

use crate::common::{assertions, test_data, NumberingConfigBuilder, TestFileManager};
use mcp_markdown_tools::tools::MarkdownToolsImpl;

#[cfg(test)]
mod tests {
  use super::*;

  /// 测试基本阿拉伯数字编号生成
  #[tokio::test]
  async fn test_generate_arabic_numbering_basic() {
    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("test.md", test_data::SIMPLE_DOC);

    let config = NumberingConfigBuilder::new(md_file.to_str().unwrap()).build();

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    assert!(result.is_ok());

    // 验证编号被正确添加
    assertions::assert_file_contains(&md_file, "# 1. 第一章 介绍");
    assertions::assert_file_contains(&md_file, "## 1.1. 背景");
    assertions::assert_file_contains(&md_file, "### 1.1.1. 历史");
    assertions::assert_file_contains(&md_file, "## 1.2. 目标");
    assertions::assert_file_contains(&md_file, "# 2. 第二章 实现");
  }

  /// 测试中文编号生成
  #[tokio::test]
  async fn test_generate_chinese_numbering() {
    let content = r#"# 第一章

## 背景

### 详细说明

## 目标

# 第二章

## 实现
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("chinese.md", content);

    let config = NumberingConfigBuilder::new(md_file.to_str().unwrap())
      .use_chinese_number(true)
      .use_arabic_for_sublevel(true)
      .build();

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    assert!(result.is_ok());

    // 验证中文编号
    assertions::assert_file_contains(&md_file, "# 一、第一章");
    assertions::assert_file_contains(&md_file, "## 1. 背景");
    assertions::assert_file_contains(&md_file, "### 1.1. 详细说明");
    assertions::assert_file_contains(&md_file, "## 2. 目标");
    assertions::assert_file_contains(&md_file, "# 二、第二章");
  }

  /// 测试忽略 H1 标题
  #[tokio::test]
  async fn test_generate_numbering_ignore_h1() {
    let content = r#"# 文档标题

前言内容

## 第一章

### 背景

## 第二章

### 实现
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("ignore_h1.md", content);

    let config = NumberingConfigBuilder::new(md_file.to_str().unwrap()).ignore_h1(true).build();

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    assert!(result.is_ok());

    // 验证 H1 未被编号，H2 和 H3 被编号
    assertions::assert_file_contains(&md_file, "# 文档标题"); // H1 保持不变
    assertions::assert_file_contains(&md_file, "## 1. 第一章");
    assertions::assert_file_contains(&md_file, "### 1.1. 背景");
    assertions::assert_file_contains(&md_file, "## 2. 第二章");
    assertions::assert_file_contains(&md_file, "### 2.1. 实现");
  }

  /// 测试编号移除功能
  #[tokio::test]
  async fn test_remove_numbering() {
    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("numbered.md", test_data::SIMPLE_DOC);

    // 先生成编号
    let generate_config = NumberingConfigBuilder::new(md_file.to_str().unwrap()).build();

    let generate_result = MarkdownToolsImpl::generate_chapter_number_impl(generate_config, "numed").await;
    assert!(generate_result.is_ok());

    // 验证编号存在
    assertions::assert_file_contains(&md_file, "# 1. 第一章 介绍");

    // 移除编号
    let remove_config = mcp_markdown_tools::config::RemoveChapterConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let remove_result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config, "unnumed").await;
    assert!(remove_result.is_ok());

    // 验证编号被移除
    assertions::assert_file_contains(&md_file, "# 第一章 介绍");
    assertions::assert_file_not_contains(&md_file, "1.");
    assertions::assert_file_not_contains(&md_file, "2.");
  }

  /// 测试保存为新文件
  #[tokio::test]
  async fn test_save_as_new_file() {
    let file_manager = TestFileManager::new();
    let original_file = file_manager.create_md_file("original.md", test_data::SIMPLE_DOC);
    let new_file_path = file_manager.temp_dir.path().join("numbered.md");

    let config = NumberingConfigBuilder::new(original_file.to_str().unwrap())
      .save_as_new_file(Some(new_file_path.to_str().unwrap().to_string()))
      .build();

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    assert!(result.is_ok());

    // 验证原文件未被修改
    assertions::assert_file_contains(&original_file, "# 第一章 介绍");
    assertions::assert_file_not_contains(&original_file, "1.");

    // 验证新文件包含编号
    assert!(new_file_path.exists());
    assertions::assert_file_contains(&new_file_path, "# 1. 第一章 介绍");
  }

  /// 测试错误处理 - 不存在的文件
  #[tokio::test]
  async fn test_error_handling_nonexistent_file() {
    let config = NumberingConfigBuilder::new("/nonexistent/file.md").build();

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    assert!(result.is_err());
  }

  /// 测试错误处理 - 非 Markdown 文件
  #[tokio::test]
  async fn test_error_handling_non_markdown_file() {
    let file_manager = TestFileManager::new();
    let txt_file = file_manager.temp_dir.path().join("test.txt");
    std::fs::write(&txt_file, "# Test").unwrap();

    let config = NumberingConfigBuilder::new(txt_file.to_str().unwrap()).build();

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    assert!(result.is_err());
  }
}
