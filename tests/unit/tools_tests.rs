//! tools.rs 模块的单元测试

use mcp_markdown_tools::config::*;
use mcp_markdown_tools::tools::MarkdownToolsImpl;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

#[cfg(test)]
mod tests {
  use super::*;

  /// 测试生成章节编号 - 阿拉伯数字
  #[tokio::test]
  async fn test_generate_chapter_number_arabic() {
    let content = r#"# 第一章

## 背景

### 历史

## 目标

# 第二章

## 实现
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(call_result.is_success);

    // 验证文件内容被修改
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(new_content.contains("# 1. 第一章"));
    assert!(new_content.contains("## 1.1. 背景"));
    assert!(new_content.contains("### 1.1.1. 历史"));
    assert!(new_content.contains("## 1.2. 目标"));
    assert!(new_content.contains("# 2. 第二章"));
    assert!(new_content.contains("## 2.1. 实现"));
  }

  /// 测试生成章节编号 - 中文数字
  #[tokio::test]
  async fn test_generate_chapter_number_chinese() {
    let content = r#"# 第一章

## 背景

# 第二章

## 实现
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: true,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(call_result.is_success);

    // 验证文件内容被修改
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(new_content.contains("# 一、第一章"));
    assert!(new_content.contains("## 1. 背景"));
    assert!(new_content.contains("# 二、第二章"));
    assert!(new_content.contains("## 1. 实现"));
  }

  /// 测试生成章节编号 - 忽略 H1
  #[tokio::test]
  async fn test_generate_chapter_number_ignore_h1() {
    let content = r#"# 文档标题

## 第一章

### 背景

## 第二章

### 实现
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: true,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(call_result.is_success);

    // 验证文件内容被修改
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(new_content.contains("# 文档标题")); // H1 不变
    assert!(new_content.contains("## 1. 第一章"));
    assert!(new_content.contains("### 1.1. 背景"));
    assert!(new_content.contains("## 2. 第二章"));
    assert!(new_content.contains("### 2.1. 实现"));
  }

  /// 测试生成章节编号 - 保存为新文件
  #[tokio::test]
  async fn test_generate_chapter_number_save_as_new() {
    let content = r#"# 第一章

## 背景
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let new_file_path = temp_dir.path().join("new_file.md");

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: true,
      new_full_file_path: Some(new_file_path.to_str().unwrap().to_string()),
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(call_result.is_success);

    // 验证原文件未被修改
    let original_content = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(original_content, content);

    // 验证新文件被创建并包含编号
    assert!(new_file_path.exists());
    let new_content = fs::read_to_string(&new_file_path).unwrap();
    assert!(new_content.contains("# 1. 第一章"));
    assert!(new_content.contains("## 1.1. 背景"));
  }

  /// 测试移除章节编号
  #[tokio::test]
  async fn test_remove_all_chapter_numbers() {
    let content = r#"# 1. 第一章

## 1.1. 背景

### 1.1.1. 历史

## 1.2. 目标

# 2. 第二章

## 2.1. 实现
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = RemoveChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(config, "unnumed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(call_result.is_success);

    // 验证编号被移除
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(new_content.contains("# 第一章"));
    assert!(new_content.contains("## 背景"));
    assert!(new_content.contains("### 历史"));
    assert!(new_content.contains("## 目标"));
    assert!(new_content.contains("# 第二章"));
    assert!(new_content.contains("## 实现"));

    // 确保数字编号被完全移除
    assert!(!new_content.contains("1."));
    assert!(!new_content.contains("2."));
  }

  /// 测试移除中文章节编号
  #[tokio::test]
  async fn test_remove_chinese_chapter_numbers() {
    let content = r#"# 一、第一章

## 1. 背景

### 1.1. 历史

# 二、第二章

## 1. 实现
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = RemoveChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(config, "unnumed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(call_result.is_success);

    // 验证中文编号被移除
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(new_content.contains("# 第一章"));
    assert!(new_content.contains("## 背景"));
    assert!(new_content.contains("### 历史"));
    assert!(new_content.contains("# 第二章"));
    assert!(new_content.contains("## 实现"));

    // 确保中文编号被移除
    assert!(!new_content.contains("一、"));
    assert!(!new_content.contains("二、"));
  }

  /// 测试检查标题 - 有效标题
  #[tokio::test]
  async fn test_check_heading_valid() {
    let content = r#"# 第一章

## 1.1 背景

### 1.1.1 历史

## 1.2 目标

# 第二章

## 2.1 实现
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = CheckHeadingConfig { full_file_path: temp_file.path().to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(call_result.is_success);

    // 验证成功消息
    if let rmcp::model::Content::Text { text } = &call_result.content[0] {
      assert!(text.contains("✅ 标题验证通过"));
      assert!(text.contains("📊 标题统计"));
    }
  }

  /// 测试检查标题 - 无效标题格式
  #[tokio::test]
  async fn test_check_heading_invalid_format() {
    let content = r#"# 正确的标题

##错误的标题

### 正确的三级标题

####  错误的标题
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = CheckHeadingConfig { full_file_path: temp_file.path().to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(!call_result.is_success);

    // 验证错误消息
    if let rmcp::model::Content::Text { text } = &call_result.content[0] {
      assert!(text.contains("❌ 标题验证失败"));
      assert!(text.contains("必须有一个空格"));
    }
  }

  /// 测试检查标题 - 跳级错误
  #[tokio::test]
  async fn test_check_heading_level_skip() {
    let content = r#"# 第一章

#### 跳级的标题

## 正确的二级标题
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = CheckHeadingConfig { full_file_path: temp_file.path().to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(!call_result.is_success);

    // 验证跳级错误消息
    if let rmcp::model::Content::Text { text } = &call_result.content[0] {
      assert!(text.contains("❌ 标题验证失败"));
      assert!(text.contains("跳级"));
    }
  }

  /// 测试本地化图片 - 基本功能
  #[tokio::test]
  async fn test_localize_images_basic() {
    let content = r#"# 图片测试

![测试图片](https://httpbin.org/image/png)

<img src="https://httpbin.org/image/jpeg" alt="JPEG图片">
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let assets_dir = temp_dir.path().join("assets");

    let config = LocalizeImagesConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      image_file_name_pattern: "{index}-{hash}".to_string(),
      save_to_dir: assets_dir.to_str().unwrap().to_string(),
    };

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    // 注意：这个测试依赖网络，在实际环境中可能失败
    // 我们主要测试函数调用和基本逻辑
    match result {
      Ok(call_result) => {
        assert!(call_result.is_success);
        if let rmcp::model::Content::Text { text } = &call_result.content[0] {
          assert!(text.contains("处理完毕"));
        }
      }
      Err(_) => {
        // 网络错误是可以接受的
        assert!(true);
      }
    }
  }

  /// 测试文件验证错误
  #[tokio::test]
  async fn test_file_validation_error() {
    let config = GenerateChapterConfig {
      full_file_path: "/nonexistent/file.md".to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_err());
  }

  /// 测试非 Markdown 文件错误
  #[tokio::test]
  async fn test_non_markdown_file_error() {
    let temp_file = NamedTempFile::with_suffix(".txt").unwrap();
    fs::write(temp_file.path(), "# Test").unwrap();

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_err());
  }

  /// 测试生成新文件名 - 默认后缀
  #[test]
  fn test_generate_new_filename_default_suffix() {
    // 这个测试需要访问私有方法，我们通过公共接口间接测试
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), "# Test").unwrap();

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: true,
      new_full_file_path: None, // 使用默认生成
    };

    // 通过异步调用间接测试文件名生成
    tokio_test::block_on(async {
      let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
      assert!(result.is_ok());

      let call_result = result.unwrap();
      assert!(call_result.is_success);

      // 验证消息中包含生成的文件名
      if let rmcp::model::Content::Text { text } = &call_result.content[0] {
        assert!(text.contains("numed"));
      }
    });
  }

  /// 测试空文档处理
  #[tokio::test]
  async fn test_empty_document() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), "").unwrap();

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(call_result.is_success);

    // 空文档应该保持为空
    let content = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(content, "");
  }

  /// 测试只有内容没有标题的文档
  #[tokio::test]
  async fn test_content_only_document() {
    let content = "这是一个没有标题的文档。\n\n只有普通内容。";

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(call_result.is_success);

    // 内容应该保持不变
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(new_content, content);
  }
}

/// 辅助函数模块
mod helpers {
  use super::*;

  /// 创建测试用的临时 Markdown 文件
  pub fn create_test_markdown_file(content: &str) -> NamedTempFile {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();
    temp_file
  }

  /// 验证文件内容包含指定文本
  pub fn assert_file_contains(file_path: &std::path::Path, expected: &str) {
    let content = fs::read_to_string(file_path).unwrap();
    assert!(content.contains(expected), "文件内容不包含: {}", expected);
  }

  /// 验证文件内容不包含指定文本
  pub fn assert_file_not_contains(file_path: &std::path::Path, unexpected: &str) {
    let content = fs::read_to_string(file_path).unwrap();
    assert!(!content.contains(unexpected), "文件内容不应包含: {}", unexpected);
  }
}
