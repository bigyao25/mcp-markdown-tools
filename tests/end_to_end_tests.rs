//! 端到端集成测试

use mcp_markdown_tools::config::*;
use mcp_markdown_tools::tools::MarkdownToolsImpl;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

#[cfg(test)]
mod tests {
  use super::*;

  /// 测试完整的编号生成工作流
  #[tokio::test]
  async fn test_complete_numbering_workflow() {
    let content = include_str!("fixtures/sample.md");

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    // 第一步：生成阿拉伯数字编号
    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    if let Err(ref e) = result {
      println!("Error: {:?}", e);
    }
    assert!(result.is_ok());

    // 验证编号被添加
    let numbered_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(numbered_content.contains("# 1. 第一章 介绍"));
    assert!(numbered_content.contains("## 1.1. 背景"));
    assert!(numbered_content.contains("### 1.1.1. 历史"));
    assert!(numbered_content.contains("## 1.2. 目标"));
    assert!(numbered_content.contains("# 2. 第二章 实现"));

    // 第二步：移除编号
    let remove_config = RemoveChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let remove_result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config, "unnumed").await;
    assert!(remove_result.is_ok());

    // 验证编号被移除，内容恢复原状
    let final_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(final_content.contains("# 第一章 介绍"));
    assert!(final_content.contains("## 背景"));
    assert!(final_content.contains("### 历史"));
    assert!(!final_content.contains("1."));
    assert!(!final_content.contains("2."));
  }

  /// 测试中文编号完整工作流
  #[tokio::test]
  async fn test_chinese_numbering_workflow() {
    let content = r#"# 第一章

## 背景

### 详细说明

## 目标

# 第二章

## 实现

### 具体步骤
"#;

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    // 生成中文编号
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

    // 验证中文编号
    let numbered_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(numbered_content.contains("# 一、第一章"));
    assert!(numbered_content.contains("## 1. 背景"));
    assert!(numbered_content.contains("### 1.1. 详细说明"));
    assert!(numbered_content.contains("## 2. 目标"));
    assert!(numbered_content.contains("# 二、第二章"));
    assert!(numbered_content.contains("## 1. 实现"));
    assert!(numbered_content.contains("### 1.1. 具体步骤"));

    // 移除编号
    let remove_config = RemoveChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let remove_result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config, "unnumed").await;
    assert!(remove_result.is_ok());

    // 验证中文编号被移除
    let final_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(final_content.contains("# 第一章"));
    assert!(!final_content.contains("一、"));
    assert!(!final_content.contains("二、"));
  }

  /// 测试忽略 H1 的完整工作流
  #[tokio::test]
  async fn test_ignore_h1_workflow() {
    let content = r#"# 文档标题

前言内容

## 第一章

### 背景

## 第二章

### 实现
"#;

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    // 生成编号，忽略 H1
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

    // 验证 H1 未被编号，H2 和 H3 被编号
    let numbered_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(numbered_content.contains("# 文档标题")); // H1 保持不变
    assert!(numbered_content.contains("## 1. 第一章"));
    assert!(numbered_content.contains("### 1.1. 背景"));
    assert!(numbered_content.contains("## 2. 第二章"));
    assert!(numbered_content.contains("### 2.1. 实现"));
  }

  /// 测试标题验证完整工作流
  #[tokio::test]
  async fn test_heading_validation_workflow() {
    // 测试有效标题
    let valid_content = r#"# 第一章

## 1.1 背景

### 1.1.1 历史

## 1.2 目标
"#;

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), valid_content).unwrap();

    let config = CheckHeadingConfig { full_file_path: temp_file.path().to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));

    // 测试无效标题
    let invalid_content = r#"# 正确标题

##错误标题

### 正确标题

####  多空格错误
"#;

    fs::write(temp_file.path(), invalid_content).unwrap();

    let config = CheckHeadingConfig { full_file_path: temp_file.path().to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));
  }

  /// 测试多文件处理工作流
  #[tokio::test]
  async fn test_multiple_files_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // 创建多个测试文件
    let files_content = vec![
      ("doc1.md", "# 文档一\n\n## 章节一\n\n### 小节一"),
      ("doc2.md", "# 文档二\n\n## 章节一\n\n## 章节二"),
      ("doc3.md", "# 文档三\n\n## 背景\n\n### 历史\n\n## 目标"),
    ];

    let mut file_paths = Vec::new();

    for (filename, content) in files_content {
      let file_path = temp_dir.path().join(filename);
      fs::write(&file_path, content).unwrap();
      file_paths.push(file_path);
    }

    // 为每个文件生成编号
    for file_path in &file_paths {
      let config = GenerateChapterConfig {
        full_file_path: file_path.to_str().unwrap().to_string(),
        ignore_h1: false,
        use_chinese_number: false,
        use_arabic_number_for_sublevel: true,
        save_as_new_file: false,
        new_full_file_path: None,
      };

      let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
      assert!(result.is_ok());
    }

    // 验证所有文件都被正确编号
    for file_path in &file_paths {
      let content = fs::read_to_string(file_path).unwrap();
      assert!(content.contains("# 1."));
      assert!(content.contains("## 1.1."));
    }

    // 移除所有文件的编号
    for file_path in &file_paths {
      let config = RemoveChapterConfig {
        full_file_path: file_path.to_str().unwrap().to_string(),
        save_as_new_file: false,
        new_full_file_path: None,
      };

      let result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(config, "unnumed").await;
      assert!(result.is_ok());
    }

    // 验证编号被移除
    for file_path in &file_paths {
      let content = fs::read_to_string(file_path).unwrap();
      assert!(!content.contains("1."));
    }
  }

  /// 测试保存为新文件的工作流
  #[tokio::test]
  async fn test_save_as_new_file_workflow() {
    let content = "# 测试文档\n\n## 第一章\n\n### 背景";

    let temp_dir = TempDir::new().unwrap();
    let original_file = temp_dir.path().join("original.md");
    let numbered_file = temp_dir.path().join("numbered.md");
    let unnumbered_file = temp_dir.path().join("unnumbered.md");

    fs::write(&original_file, content).unwrap();

    // 生成编号并保存为新文件
    let config = GenerateChapterConfig {
      full_file_path: original_file.to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: true,
      new_full_file_path: Some(numbered_file.to_str().unwrap().to_string()),
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    assert!(result.is_ok());

    // 验证原文件未被修改
    let original_content = fs::read_to_string(&original_file).unwrap();
    assert_eq!(original_content, content);

    // 验证新文件包含编号
    assert!(numbered_file.exists());
    let numbered_content = fs::read_to_string(&numbered_file).unwrap();
    assert!(numbered_content.contains("# 1. 测试文档"));
    assert!(numbered_content.contains("## 1.1. 第一章"));
    assert!(numbered_content.contains("### 1.1.1. 背景"));

    // 从编号文件移除编号并保存为另一个新文件
    let remove_config = RemoveChapterConfig {
      full_file_path: numbered_file.to_str().unwrap().to_string(),
      save_as_new_file: true,
      new_full_file_path: Some(unnumbered_file.to_str().unwrap().to_string()),
    };

    let remove_result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config, "unnumed").await;
    assert!(remove_result.is_ok());

    // 验证编号文件未被修改
    let still_numbered_content = fs::read_to_string(&numbered_file).unwrap();
    assert!(still_numbered_content.contains("1."));

    // 验证新的无编号文件
    assert!(unnumbered_file.exists());
    let final_content = fs::read_to_string(&unnumbered_file).unwrap();
    assert!(final_content.contains("# 测试文档"));
    assert!(!final_content.contains("1."));
  }

  /// 测试复杂文档结构的处理
  #[tokio::test]
  async fn test_complex_document_structure() {
    let content = include_str!("fixtures/complex.md");

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    // 先验证标题结构
    let check_config = CheckHeadingConfig { full_file_path: temp_file.path().to_str().unwrap().to_string() };

    let check_result = MarkdownToolsImpl::check_heading_impl(check_config).await;
    assert!(check_result.is_ok());

    // 生成中文编号
    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: true,
      use_arabic_number_for_sublevel: false,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    assert!(result.is_ok());

    // 验证复杂结构的编号
    let numbered_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(numbered_content.contains("# 一、主标题"));
    assert!(numbered_content.contains("## 一、一、子标题"));
    assert!(numbered_content.contains("### 一、一、一、三级标题"));
    assert!(numbered_content.contains("# 二、第二个主标题"));
  }

  /// 测试错误恢复工作流
  #[tokio::test]
  async fn test_error_recovery_workflow() {
    // 测试不存在的文件
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

    // 测试非 Markdown 文件
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

  /// 测试性能 - 大文档处理
  #[tokio::test]
  async fn test_large_document_performance() {
    // 生成一个较大的文档
    let mut large_content = String::new();
    for i in 1..=100 {
      large_content.push_str(&format!("# 第{}章\n\n", i));
      for j in 1..=5 {
        large_content.push_str(&format!("## {}.{} 小节\n\n", i, j));
        for k in 1..=3 {
          large_content.push_str(&format!("### {}.{}.{} 子小节\n\n内容...\n\n", i, j, k));
        }
      }
    }

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), &large_content).unwrap();

    let start_time = std::time::Instant::now();

    // 生成编号
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

    let duration = start_time.elapsed();

    // 验证处理时间合理（应该在几秒内完成）
    assert!(duration.as_secs() < 10, "处理时间过长: {:?}", duration);

    // 验证结果正确性
    let numbered_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(numbered_content.contains("# 1. 第1章"));
    assert!(numbered_content.contains("# 100. 第100章"));
    assert!(numbered_content.contains("## 1.1. 小节"));
    assert!(numbered_content.contains("### 100.5.3. 子小节"));
  }
}
