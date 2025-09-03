//! 标题验证功能单元测试
//!
//! 测试 Markdown 标题格式和层级结构验证的核心逻辑

use crate::common::{assertions, TestFileManager};
use mcp_markdown_tools::config::CheckHeadingConfig;
use mcp_markdown_tools::tools::MarkdownToolsImpl;

#[cfg(test)]
mod tests {
  use super::*;

  /// 测试有效标题验证
  #[tokio::test]
  async fn test_validate_heading_valid_structure() {
    let valid_content = r#"# 第一章

## 1.1 背景

### 1.1.1 历史

## 1.2 目标

# 第二章

## 2.1 实现
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("valid.md", valid_content);

    let config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));
  }

  /// 测试无效标题格式 - 缺少空格
  #[tokio::test]
  async fn test_validate_heading_missing_space() {
    let invalid_content = r#"# 正确标题

##错误标题

### 正确标题
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("invalid_space.md", invalid_content);

    let config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));
  }

  /// 测试无效标题格式 - 多余空格
  #[tokio::test]
  async fn test_validate_heading_extra_spaces() {
    let invalid_content = r#"# 正确标题

##  多空格错误

### 正确标题

####   更多空格错误
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("invalid_extra_spaces.md", invalid_content);

    let config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));
  }

  /// 测试标题层级跳跃
  #[tokio::test]
  async fn test_validate_heading_level_jumping() {
    let invalid_content = r#"# 第一章

### 跳级错误（应该是 H2）

## 正确的 H2

##### 又跳级了（应该是 H3 或 H4）
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("level_jumping.md", invalid_content);

    let config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));
  }

  /// 测试复杂但有效的标题结构
  #[tokio::test]
  async fn test_validate_heading_complex_valid() {
    let valid_content = r#"# 主标题

前言内容，不是标题。

## 第一章

### 1.1 背景

#### 1.1.1 历史背景

##### 1.1.1.1 详细历史

###### 1.1.1.1.1 更详细

### 1.2 目标

## 第二章

### 2.1 实现

# 第二部分

## 新的开始
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("complex_valid.md", valid_content);

    let config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));
  }

  /// 测试空文档
  #[tokio::test]
  async fn test_validate_heading_empty_document() {
    let empty_content = "";

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("empty.md", empty_content);

    let config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    // 空文档应该被认为是有效的
    assert_eq!(call_result.is_error, Some(false));
  }

  /// 测试只有内容没有标题的文档
  #[tokio::test]
  async fn test_validate_heading_no_headings() {
    let content_only = r#"这是一个没有标题的文档。

只有普通的段落内容。

还有更多内容。
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("no_headings.md", content_only);

    let config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    // 没有标题的文档应该被认为是有效的
    assert_eq!(call_result.is_error, Some(false));
  }

  /// 测试错误处理 - 不存在的文件
  #[tokio::test]
  async fn test_validate_heading_nonexistent_file() {
    let config = CheckHeadingConfig { full_file_path: "/nonexistent/file.md".to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));
  }

  /// 测试文档开头有非标题内容
  #[tokio::test]
  async fn test_validate_heading_with_preamble() {
    let content_with_preamble = r#"这是文档的前言部分。

可能包含一些说明文字。

# 第一章 正式开始

## 1.1 小节

### 1.1.1 子小节

## 1.2 另一个小节

# 第二章

## 2.1 实现
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("with_preamble.md", content_with_preamble);

    let config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));
  }
}
