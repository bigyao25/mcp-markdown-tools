//! 编号功能集成测试
//!
//! 测试编号功能与其他模块的集成，包括解析器、验证器等的协同工作

use crate::common::{assertions, test_data, NumberingConfigBuilder, TestFileManager};
use mcp_markdown_tools::config::{CheckHeadingConfig, GenerateChapterConfig, RemoveChapterConfig};
use mcp_markdown_tools::tools::MarkdownToolsImpl;

#[cfg(test)]
mod tests {
  use super::*;

  /// 集成测试：编号生成 + 验证 + 移除的完整流程
  #[tokio::test]
  async fn integration_complete_numbering_workflow() {
    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("workflow.md", test_data::SIMPLE_DOC);

    // 第一步：验证原始文档结构
    let check_config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let check_result = MarkdownToolsImpl::check_heading_impl(check_config).await;
    assert!(check_result.is_ok());
    assert_eq!(check_result.unwrap().is_error, Some(false));

    // 第二步：生成阿拉伯数字编号
    let generate_config = NumberingConfigBuilder::new(md_file.to_str().unwrap()).build();

    let generate_result = MarkdownToolsImpl::generate_chapter_number_impl(generate_config, "numed").await;
    assert!(generate_result.is_ok());

    // 验证编号被添加
    assertions::assert_file_contains(&md_file, "# 1. 第一章 介绍");
    assertions::assert_file_contains(&md_file, "## 1.1. 背景");
    assertions::assert_file_contains(&md_file, "### 1.1.1. 历史");
    assertions::assert_file_contains(&md_file, "## 1.2. 目标");
    assertions::assert_file_contains(&md_file, "# 2. 第二章 实现");

    // 第三步：验证编号后的文档结构仍然有效
    let check_config_after = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let check_result_after = MarkdownToolsImpl::check_heading_impl(check_config_after).await;
    assert!(check_result_after.is_ok());
    assert_eq!(check_result_after.unwrap().is_error, Some(false));

    // 第四步：移除编号
    let remove_config = RemoveChapterConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let remove_result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config, "unnumed").await;
    assert!(remove_result.is_ok());

    // 验证编号被移除，内容恢复原状
    assertions::assert_file_contains(&md_file, "# 第一章 介绍");
    assertions::assert_file_contains(&md_file, "## 背景");
    assertions::assert_file_contains(&md_file, "### 历史");
    assertions::assert_file_not_contains(&md_file, "1.");
    assertions::assert_file_not_contains(&md_file, "2.");

    // 第五步：验证移除编号后的文档结构仍然有效
    let check_config_final = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let check_result_final = MarkdownToolsImpl::check_heading_impl(check_config_final).await;
    assert!(check_result_final.is_ok());
    assert_eq!(check_result_final.unwrap().is_error, Some(false));
  }

  /// 集成测试：中文编号完整工作流
  #[tokio::test]
  async fn integration_chinese_numbering_workflow() {
    let content = r#"# 第一章

## 背景

### 详细说明

## 目标

# 第二章

## 实现

### 具体步骤
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("chinese_workflow.md", content);

    // 生成中文编号
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
    assertions::assert_file_contains(&md_file, "## 1. 实现");
    assertions::assert_file_contains(&md_file, "### 1.1. 具体步骤");

    // 验证编号后的结构
    let check_config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let check_result = MarkdownToolsImpl::check_heading_impl(check_config).await;
    assert!(check_result.is_ok());
    assert_eq!(check_result.unwrap().is_error, Some(false));

    // 移除编号
    let remove_config = RemoveChapterConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let remove_result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config, "unnumed").await;
    assert!(remove_result.is_ok());

    // 验证中文编号被移除
    assertions::assert_file_contains(&md_file, "# 第一章");
    assertions::assert_file_not_contains(&md_file, "一、");
    assertions::assert_file_not_contains(&md_file, "二、");
  }

  /// 集成测试：忽略 H1 的完整工作流
  #[tokio::test]
  async fn integration_ignore_h1_workflow() {
    let content = r#"# 文档标题

前言内容

## 第一章

### 背景

## 第二章

### 实现
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("ignore_h1_workflow.md", content);

    // 生成编号，忽略 H1
    let config = NumberingConfigBuilder::new(md_file.to_str().unwrap()).ignore_h1(true).build();

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    assert!(result.is_ok());

    // 验证 H1 未被编号，H2 和 H3 被编号
    assertions::assert_file_contains(&md_file, "# 文档标题"); // H1 保持不变
    assertions::assert_file_contains(&md_file, "## 1. 第一章");
    assertions::assert_file_contains(&md_file, "### 1.1. 背景");
    assertions::assert_file_contains(&md_file, "## 2. 第二章");
    assertions::assert_file_contains(&md_file, "### 2.1. 实现");

    // 验证结构仍然有效
    let check_config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let check_result = MarkdownToolsImpl::check_heading_impl(check_config).await;
    assert!(check_result.is_ok());
    assert_eq!(check_result.unwrap().is_error, Some(false));
  }

  /// 集成测试：多文件处理工作流
  #[tokio::test]
  async fn integration_multiple_files_workflow() {
    let file_manager = TestFileManager::new();

    // 创建多个测试文件
    let files_content = vec![
      ("doc1.md", "# 文档一\n\n## 章节一\n\n### 小节一"),
      ("doc2.md", "# 文档二\n\n## 章节一\n\n## 章节二"),
      ("doc3.md", "# 文档三\n\n## 背景\n\n### 历史\n\n## 目标"),
    ];

    let mut file_paths = Vec::new();

    for (filename, content) in files_content {
      let file_path = file_manager.create_md_file(filename, content);
      file_paths.push(file_path);
    }

    // 为每个文件生成编号
    for file_path in &file_paths {
      let config = NumberingConfigBuilder::new(file_path.to_str().unwrap()).build();

      let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
      assert!(result.is_ok());
    }

    // 验证所有文件都被正确编号
    for file_path in &file_paths {
      assertions::assert_file_contains(file_path, "# 1.");
      assertions::assert_file_contains(file_path, "## 1.1.");
    }

    // 验证所有文件的结构都有效
    for file_path in &file_paths {
      let check_config = CheckHeadingConfig { full_file_path: file_path.to_str().unwrap().to_string() };

      let check_result = MarkdownToolsImpl::check_heading_impl(check_config).await;
      assert!(check_result.is_ok());
      assert_eq!(check_result.unwrap().is_error, Some(false));
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
      assertions::assert_file_not_contains(file_path, "1.");
    }
  }

  /// 集成测试：保存为新文件的工作流
  #[tokio::test]
  async fn integration_save_as_new_file_workflow() {
    let content = "# 测试文档\n\n## 第一章\n\n### 背景";

    let file_manager = TestFileManager::new();
    let original_file = file_manager.create_md_file("original.md", content);
    let numbered_file = file_manager.temp_dir.path().join("numbered.md");
    let unnumbered_file = file_manager.temp_dir.path().join("unnumbered.md");

    // 生成编号并保存为新文件
    let config = NumberingConfigBuilder::new(original_file.to_str().unwrap())
      .save_as_new_file(Some(numbered_file.to_str().unwrap().to_string()))
      .build();

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    assert!(result.is_ok());

    // 验证原文件未被修改
    assertions::assert_file_contains(&original_file, "# 测试文档");
    assertions::assert_file_not_contains(&original_file, "1.");

    // 验证新文件包含编号
    assert!(numbered_file.exists());
    assertions::assert_file_contains(&numbered_file, "# 1. 测试文档");
    assertions::assert_file_contains(&numbered_file, "## 1.1. 第一章");
    assertions::assert_file_contains(&numbered_file, "### 1.1.1. 背景");

    // 验证编号文件的结构
    let check_config = CheckHeadingConfig { full_file_path: numbered_file.to_str().unwrap().to_string() };

    let check_result = MarkdownToolsImpl::check_heading_impl(check_config).await;
    assert!(check_result.is_ok());
    assert_eq!(check_result.unwrap().is_error, Some(false));

    // 从编号文件移除编号并保存为另一个新文件
    let remove_config = RemoveChapterConfig {
      full_file_path: numbered_file.to_str().unwrap().to_string(),
      save_as_new_file: true,
      new_full_file_path: Some(unnumbered_file.to_str().unwrap().to_string()),
    };

    let remove_result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config, "unnumed").await;
    assert!(remove_result.is_ok());

    // 验证编号文件未被修改
    assertions::assert_file_contains(&numbered_file, "1.");

    // 验证新的无编号文件
    assert!(unnumbered_file.exists());
    assertions::assert_file_contains(&unnumbered_file, "# 测试文档");
    assertions::assert_file_not_contains(&unnumbered_file, "1.");
  }

  /// 集成测试：复杂文档结构的处理
  #[tokio::test]
  async fn integration_complex_document_structure() {
    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("complex.md", test_data::COMPLEX_DOC);

    // 先验证标题结构
    let check_config = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let check_result = MarkdownToolsImpl::check_heading_impl(check_config).await;
    assert!(check_result.is_ok());

    // 生成中文编号
    let config = NumberingConfigBuilder::new(md_file.to_str().unwrap())
      .use_chinese_number(true)
      .use_arabic_for_sublevel(false)
      .build();

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
    assert!(result.is_ok());

    // 验证复杂结构的编号
    assertions::assert_file_contains(&md_file, "# 一、主标题");
    assertions::assert_file_contains(&md_file, "## 一、一、子标题");
    assertions::assert_file_contains(&md_file, "### 一、一、一、三级标题");
    assertions::assert_file_contains(&md_file, "# 二、第二个主标题");

    // 验证编号后结构仍然有效
    let check_config_after = CheckHeadingConfig { full_file_path: md_file.to_str().unwrap().to_string() };

    let check_result_after = MarkdownToolsImpl::check_heading_impl(check_config_after).await;
    assert!(check_result_after.is_ok());
    assert_eq!(check_result_after.unwrap().is_error, Some(false));
  }
}
