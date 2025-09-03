//! 端到端工作流测试
//!
//! 测试完整的用户使用场景和工作流程

use crate::common::{assertions, test_data, ImageLocalizationConfigBuilder, NumberingConfigBuilder, TestFileManager};
use mcp_markdown_tools::config::{GenerateChapterConfig, LocalizeImagesConfig, RemoveChapterConfig};
use mcp_markdown_tools::tools::MarkdownToolsImpl;

#[cfg(test)]
mod tests {
  use super::*;

  /// 端到端测试：完整的文档处理工作流
  /// 场景：用户创建文档 -> 生成编号 -> 本地化图片 -> 移除编号
  #[tokio::test]
  async fn e2e_complete_document_workflow() {
    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("complete_workflow.md", test_data::SIMPLE_DOC);
    let assets_dir = file_manager.assets_dir();

    // 第一步：生成章节编号
    let numbering_config = NumberingConfigBuilder::new(md_file.to_str().unwrap()).build();

    let numbering_result = MarkdownToolsImpl::generate_chapter_number_impl(numbering_config, "numed").await;
    assert!(numbering_result.is_ok());

    // 验证编号被添加
    assertions::assert_file_contains(&md_file, "# 1. 第一章 介绍");
    assertions::assert_file_contains(&md_file, "## 1.1. 背景");

    // 第二步：本地化图片（虽然这个文档没有图片）
    let localization_config =
      ImageLocalizationConfigBuilder::new(md_file.to_str().unwrap()).save_to_dir(assets_dir.to_str().unwrap()).build();

    let localization_result = MarkdownToolsImpl::localize_images_impl(localization_config).await;
    assert!(localization_result.is_ok());

    let call_result = localization_result.unwrap();
    assert_eq!(call_result.is_error, Some(false));

    // 验证资源目录被创建
    assertions::assert_dir_exists(&assets_dir);

    // 第三步：移除编号
    let remove_config = RemoveChapterConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let remove_result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config, "unnumed").await;
    assert!(remove_result.is_ok());

    // 验证编号被移除
    assertions::assert_file_contains(&md_file, "# 第一章 介绍");
    assertions::assert_file_not_contains(&md_file, "1.");
  }

  /// 端到端测试：多语言编号工作流
  /// 场景：用户需要中文编号 -> 转换为阿拉伯数字 -> 再转回中文
  #[tokio::test]
  async fn e2e_multilingual_numbering_workflow() {
    let content = r#"# 第一章 概述

## 背景介绍

### 历史沿革

## 目标设定

# 第二章 实施

## 技术方案

### 具体步骤
"#;

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("multilingual.md", content);

    // 第一步：生成中文编号
    let chinese_config = NumberingConfigBuilder::new(md_file.to_str().unwrap())
      .use_chinese_number(true)
      .use_arabic_for_sublevel(true)
      .build();

    let chinese_result = MarkdownToolsImpl::generate_chapter_number_impl(chinese_config, "numed").await;
    assert!(chinese_result.is_ok());

    // 验证中文编号
    assertions::assert_file_contains(&md_file, "# 一、第一章 概述");
    assertions::assert_file_contains(&md_file, "## 1. 背景介绍");
    assertions::assert_file_contains(&md_file, "### 1.1. 历史沿革");

    // 第二步：移除编号
    let remove_config = RemoveChapterConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let remove_result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config, "unnumed").await;
    assert!(remove_result.is_ok());

    // 第三步：生成阿拉伯数字编号
    let arabic_config = NumberingConfigBuilder::new(md_file.to_str().unwrap()).use_chinese_number(false).build();

    let arabic_result = MarkdownToolsImpl::generate_chapter_number_impl(arabic_config, "numed").await;
    assert!(arabic_result.is_ok());

    // 验证阿拉伯数字编号
    assertions::assert_file_contains(&md_file, "# 1. 第一章 概述");
    assertions::assert_file_contains(&md_file, "## 1.1. 背景介绍");
    assertions::assert_file_contains(&md_file, "### 1.1.1. 历史沿革");

    // 第四步：再次移除编号
    let remove_config2 = RemoveChapterConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let remove_result2 = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config2, "unnumed").await;
    assert!(remove_result2.is_ok());

    // 第五步：再次生成中文编号（但这次不使用阿拉伯数字子级别）
    let chinese_config2 = NumberingConfigBuilder::new(md_file.to_str().unwrap())
      .use_chinese_number(true)
      .use_arabic_for_sublevel(false)
      .build();

    let chinese_result2 = MarkdownToolsImpl::generate_chapter_number_impl(chinese_config2, "numed").await;
    assert!(chinese_result2.is_ok());

    // 验证全中文编号
    assertions::assert_file_contains(&md_file, "# 一、第一章 概述");
    assertions::assert_file_contains(&md_file, "## 一、一、背景介绍");
    assertions::assert_file_contains(&md_file, "### 一、一、一、历史沿革");
  }

  /// 端到端测试：文档版本管理工作流
  /// 场景：用户需要保留原文档，创建多个版本
  #[tokio::test]
  async fn e2e_document_versioning_workflow() {
    let file_manager = TestFileManager::new();
    let original_file = file_manager.create_md_file("original.md", test_data::SIMPLE_DOC);

    // 定义版本文件路径
    let v1_numbered = file_manager.temp_dir.path().join("v1_numbered.md");
    let v2_chinese = file_manager.temp_dir.path().join("v2_chinese.md");
    let v3_ignore_h1 = file_manager.temp_dir.path().join("v3_ignore_h1.md");

    // 版本 1：阿拉伯数字编号
    let v1_config = NumberingConfigBuilder::new(original_file.to_str().unwrap())
      .save_as_new_file(Some(v1_numbered.to_str().unwrap().to_string()))
      .build();

    let v1_result = MarkdownToolsImpl::generate_chapter_number_impl(v1_config, "numed").await;
    assert!(v1_result.is_ok());

    // 版本 2：中文编号
    let v2_config = NumberingConfigBuilder::new(original_file.to_str().unwrap())
      .use_chinese_number(true)
      .save_as_new_file(Some(v2_chinese.to_str().unwrap().to_string()))
      .build();

    let v2_result = MarkdownToolsImpl::generate_chapter_number_impl(v2_config, "numed").await;
    assert!(v2_result.is_ok());

    // 版本 3：忽略 H1 的编号
    let v3_config = NumberingConfigBuilder::new(original_file.to_str().unwrap())
      .ignore_h1(true)
      .save_as_new_file(Some(v3_ignore_h1.to_str().unwrap().to_string()))
      .build();

    let v3_result = MarkdownToolsImpl::generate_chapter_number_impl(v3_config, "numed").await;
    assert!(v3_result.is_ok());

    // 验证原文档未被修改
    assertions::assert_file_contains(&original_file, "# 第一章 介绍");
    assertions::assert_file_not_contains(&original_file, "1.");

    // 验证版本 1
    assert!(v1_numbered.exists());
    assertions::assert_file_contains(&v1_numbered, "# 1. 第一章 介绍");

    // 验证版本 2
    assert!(v2_chinese.exists());
    assertions::assert_file_contains(&v2_chinese, "# 一、第一章 介绍");

    // 验证版本 3
    assert!(v3_ignore_h1.exists());
    assertions::assert_file_contains(&v3_ignore_h1, "# 第一章 介绍"); // H1 未编号
    assertions::assert_file_contains(&v3_ignore_h1, "## 1. 背景"); // H2 开始编号
  }

  /// 端到端测试：批量文档处理工作流
  /// 场景：用户需要处理多个文档文件
  #[tokio::test]
  async fn e2e_batch_processing_workflow() {
    let file_manager = TestFileManager::new();

    // 创建多个不同类型的文档
    let documents = vec![
      ("simple.md", test_data::SIMPLE_DOC),
      ("complex.md", test_data::COMPLEX_DOC),
      ("no_images.md", test_data::DOC_WITHOUT_IMAGES),
    ];

    let mut file_paths = Vec::new();

    // 创建所有文档
    for (filename, content) in &documents {
      let file_path = file_manager.create_md_file(filename, content);
      file_paths.push(file_path);
    }

    // 第一阶段：为所有文档生成编号
    for file_path in &file_paths {
      let config = NumberingConfigBuilder::new(file_path.to_str().unwrap()).build();

      let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;
      assert!(result.is_ok(), "文件 {:?} 编号生成失败", file_path);
    }

    // 验证所有文档都有编号
    for file_path in &file_paths {
      assertions::assert_file_contains(file_path, "# 1.");
    }

    // 第二阶段：为所有文档本地化图片
    let assets_dir = file_manager.assets_dir();
    for file_path in &file_paths {
      let config = ImageLocalizationConfigBuilder::new(file_path.to_str().unwrap())
        .save_to_dir(assets_dir.to_str().unwrap())
        .build();

      let result = MarkdownToolsImpl::localize_images_impl(config).await;
      assert!(result.is_ok(), "文件 {:?} 图片本地化失败", file_path);
    }

    // 验证资源目录被创建
    assertions::assert_dir_exists(&assets_dir);

    // 第三阶段：移除所有文档的编号
    for file_path in &file_paths {
      let config = RemoveChapterConfig {
        full_file_path: file_path.to_str().unwrap().to_string(),
        save_as_new_file: false,
        new_full_file_path: None,
      };

      let result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(config, "unnumed").await;
      assert!(result.is_ok(), "文件 {:?} 编号移除失败", file_path);
    }

    // 验证所有编号都被移除
    for file_path in &file_paths {
      assertions::assert_file_not_contains(file_path, "1.");
    }
  }

  /// 端到端测试：错误恢复和用户体验工作流
  /// 场景：用户在操作过程中遇到各种错误，系统应该优雅处理
  #[tokio::test]
  async fn e2e_error_recovery_workflow() {
    let file_manager = TestFileManager::new();

    // 场景 1：处理不存在的文件
    let nonexistent_config = NumberingConfigBuilder::new("/nonexistent/file.md").build();

    let nonexistent_result = MarkdownToolsImpl::generate_chapter_number_impl(nonexistent_config, "numed").await;
    assert!(nonexistent_result.is_err(), "应该报告文件不存在错误");

    // 场景 2：处理非 Markdown 文件
    let txt_file = file_manager.temp_dir.path().join("test.txt");
    std::fs::write(&txt_file, "# Test").unwrap();

    let txt_config = NumberingConfigBuilder::new(txt_file.to_str().unwrap()).build();

    let txt_result = MarkdownToolsImpl::generate_chapter_number_impl(txt_config, "numed").await;
    assert!(txt_result.is_err(), "应该报告非 Markdown 文件错误");

    // 场景 3：处理有效文档，验证系统仍然正常工作
    let valid_file = file_manager.create_md_file("valid.md", test_data::SIMPLE_DOC);

    let valid_config = NumberingConfigBuilder::new(valid_file.to_str().unwrap()).build();

    let valid_result = MarkdownToolsImpl::generate_chapter_number_impl(valid_config, "numed").await;
    assert!(valid_result.is_ok(), "有效文档处理应该成功");

    // 验证有效文档被正确处理
    assertions::assert_file_contains(&valid_file, "# 1. 第一章 介绍");

    // 场景 4：图片本地化错误处理
    let img_config = ImageLocalizationConfigBuilder::new("/nonexistent/image_file.md").build();

    let img_result = MarkdownToolsImpl::localize_images_impl(img_config).await;
    // 图片本地化可能返回 Ok 但标记错误，或直接返回 Err
    match img_result {
      Ok(call_result) => assert_eq!(call_result.is_error, Some(true)),
      Err(_) => assert!(true), // 直接错误也可接受
    }

    // 验证系统在错误后仍能正常工作
    let valid_img_config = ImageLocalizationConfigBuilder::new(valid_file.to_str().unwrap())
      .save_to_dir(file_manager.assets_dir().to_str().unwrap())
      .build();

    let valid_img_result = MarkdownToolsImpl::localize_images_impl(valid_img_config).await;
    assert!(valid_img_result.is_ok(), "错误恢复后应该能正常处理");
  }

  /// 端到端测试：性能和大规模处理工作流
  /// 场景：用户需要处理大型文档或大量文档
  #[tokio::test]
  async fn e2e_performance_workflow() {
    let file_manager = TestFileManager::new();

    // 生成一个大型文档
    let mut large_content = String::new();
    for i in 1..=50 {
      large_content.push_str(&format!("# 第{}章\n\n", i));
      for j in 1..=10 {
        large_content.push_str(&format!("## {}.{} 小节\n\n", i, j));
        for k in 1..=5 {
          large_content.push_str(&format!("### {}.{}.{} 子小节\n\n内容...\n\n", i, j, k));
        }
      }
    }

    let large_file = file_manager.create_md_file("large.md", &large_content);

    let start_time = std::time::Instant::now();

    // 第一步：生成编号
    let numbering_config = NumberingConfigBuilder::new(large_file.to_str().unwrap()).build();

    let numbering_result = MarkdownToolsImpl::generate_chapter_number_impl(numbering_config, "numed").await;
    assert!(numbering_result.is_ok());

    let numbering_duration = start_time.elapsed();

    // 第二步：本地化图片
    let localization_config = ImageLocalizationConfigBuilder::new(large_file.to_str().unwrap())
      .save_to_dir(file_manager.assets_dir().to_str().unwrap())
      .build();

    let localization_result = MarkdownToolsImpl::localize_images_impl(localization_config).await;
    assert!(localization_result.is_ok());

    let localization_duration = start_time.elapsed();

    // 第三步：移除编号
    let remove_config = RemoveChapterConfig {
      full_file_path: large_file.to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let remove_result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config, "unnumed").await;
    assert!(remove_result.is_ok());

    let total_duration = start_time.elapsed();

    // 验证性能合理（具体时间取决于系统性能）
    assert!(numbering_duration.as_secs() < 30, "编号生成时间过长: {:?}", numbering_duration);
    assert!(localization_duration.as_secs() < 35, "图片本地化时间过长: {:?}", localization_duration);
    assert!(total_duration.as_secs() < 60, "总处理时间过长: {:?}", total_duration);

    // 验证结果正确性
    assertions::assert_file_contains(&large_file, "# 第1章");
    assertions::assert_file_contains(&large_file, "# 第50章");
    assertions::assert_file_not_contains(&large_file, "1.");
  }
}

#[cfg(feature = "mock")]
mod mock_e2e_tests {
  use super::*;
  use crate::common::MockHttpServer;
  use rstest::rstest;

  #[rstest::fixture]
  async fn mock_server() -> MockHttpServer {
    let server = MockHttpServer::new().await;
    server.mock_multiple_images(5).await;
    server
  }

  /// 端到端测试：编号生成 + 图片本地化的组合工作流（使用 Mock）
  #[rstest]
  #[tokio::test]
  async fn e2e_combined_numbering_and_localization_workflow(#[future] mock_server: MockHttpServer) {
    let server = mock_server.await;
    let host = server.url();
    let content = format!(
      r#"# 组合测试文档

## 第一章

这里有一张图片：

![章节图片]({0}/image1.png)

### 1.1 小节

另一张图片：

<img src="{0}/image2.svg" alt="小节图片">

## 第二章

第二章的图片：

![第二章图片]({0}/image3.png "第二章")
"#,
      host
    );

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("combined.md", &content);
    let assets_dir = file_manager.assets_dir();

    // 第一步：生成章节编号
    let numbering_config = NumberingConfigBuilder::new(md_file.to_str().unwrap()).build();

    let numbering_result = MarkdownToolsImpl::generate_chapter_number_impl(numbering_config, "numed").await;
    assert!(numbering_result.is_ok());

    // 验证编号被添加
    assertions::assert_file_contains(&md_file, "# 1. 组合测试文档");
    assertions::assert_file_contains(&md_file, "## 1.1. 第一章");

    // 第二步：本地化图片
    let localization_config =
      ImageLocalizationConfigBuilder::new(md_file.to_str().unwrap()).save_to_dir(assets_dir.to_str().unwrap()).build();

    let localization_result = MarkdownToolsImpl::localize_images_impl(localization_config).await;

    match localization_result {
      Ok(call_result) => {
        assert_eq!(call_result.is_error, Some(false));
        assertions::assert_dir_exists(&assets_dir);

        // 验证图片被下载
        let assets_files: Vec<_> = std::fs::read_dir(&assets_dir)
          .unwrap()
          .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
          .collect();

        assert!(!assets_files.is_empty(), "应该有图片文件被下载");

        // 验证 Markdown 文件中的 URL 已更新
        let final_content = std::fs::read_to_string(&md_file).unwrap();
        assert!(final_content.contains("assets/"));
        assert!(!final_content.contains(&host)); // 不应该再包含原始 URL

        // 验证编号仍然存在
        assert!(final_content.contains("# 1. 组合测试文档"));
      }
      Err(e) => panic!("组合工作流不应该失败: {:?}", e),
    }

    // 第三步：移除编号（保留本地化的图片）
    let remove_config = RemoveChapterConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let remove_result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(remove_config, "unnumed").await;
    assert!(remove_result.is_ok());

    // 验证编号被移除但图片链接保持本地化
    let final_content = std::fs::read_to_string(&md_file).unwrap();
    assertions::assert_file_contains(&md_file, "# 组合测试文档");
    assertions::assert_file_not_contains(&md_file, "1.");
    assert!(final_content.contains("assets/")); // 图片仍然是本地化的
  }
}
