//! 图片本地化集成测试
//!
//! 测试图片本地化功能与其他模块的集成，包括解析器、网络请求等的协同工作

use crate::common::{assertions, test_data, ImageLocalizationConfigBuilder, MockHttpServer, TestFileManager};
use mcp_markdown_tools::config::LocalizeImagesConfig;
use mcp_markdown_tools::tools::MarkdownToolsImpl;

#[cfg(test)]
mod tests {
  use super::*;

  /// 集成测试：无图片文档处理
  #[tokio::test]
  async fn integration_no_images_document() {
    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("no_images.md", test_data::DOC_WITHOUT_IMAGES);
    let assets_dir = file_manager.assets_dir();

    let config =
      ImageLocalizationConfigBuilder::new(md_file.to_str().unwrap()).save_to_dir(assets_dir.to_str().unwrap()).build();

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));

    // 验证目录被创建
    assertions::assert_dir_exists(&assets_dir);

    // 验证文档内容未被修改
    let final_content = std::fs::read_to_string(&md_file).unwrap();
    assert_eq!(final_content, test_data::DOC_WITHOUT_IMAGES);
  }

  /// 集成测试：错误处理 - 不存在的文件
  #[tokio::test]
  async fn integration_error_handling_nonexistent_file() {
    let config = ImageLocalizationConfigBuilder::new("/nonexistent/file.md").build();

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));
    assert!(!call_result.content.is_empty());
  }

  /// 集成测试：非 Markdown 文件处理
  #[tokio::test]
  async fn integration_non_markdown_file() {
    let file_manager = TestFileManager::new();
    let txt_file = file_manager.temp_dir.path().join("test.txt");
    std::fs::write(&txt_file, "![图片](https://example.com/test.jpg)").unwrap();

    let config = ImageLocalizationConfigBuilder::new(txt_file.to_str().unwrap())
      .save_to_dir(file_manager.assets_dir().to_str().unwrap())
      .build();

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));
    assert!(!call_result.content.is_empty());
  }

  /// 集成测试：占位符解析
  #[tokio::test]
  async fn integration_placeholder_resolution() {
    let file_manager = TestFileManager::new();
    let docs_dir = file_manager.temp_dir.path().join("docs");
    std::fs::create_dir_all(&docs_dir).unwrap();

    let md_file = docs_dir.join("test.md");
    std::fs::write(&md_file, "# 测试文档\n\n无图片内容。").unwrap();

    // 使用占位符配置
    let config = LocalizeImagesConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      image_file_name_pattern: "{multilevel_num}-{index}".to_string(),
      save_to_dir: "{full_dir_of_original_file}/assets/".to_string(),
      new_full_file_path: None,
    };

    // 验证占位符解析
    let resolved_dir = config.get_resolved_save_dir();
    let expected_dir = docs_dir.join("assets").to_str().unwrap().to_string() + "/";
    assert_eq!(resolved_dir, expected_dir);

    // 执行本地化
    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    // 验证目录结构
    let assets_dir = docs_dir.join("assets");
    match result {
      Ok(_) => assertions::assert_dir_exists(&assets_dir),
      Err(_) => panic!("应该成功处理无图片文档"),
    }
  }

  /// 集成测试：自定义文件名模式
  #[tokio::test]
  async fn integration_custom_filename_pattern() {
    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("custom.md", test_data::DOC_WITHOUT_IMAGES);
    let images_dir = file_manager.temp_dir.path().join("images");

    let config = ImageLocalizationConfigBuilder::new(md_file.to_str().unwrap())
      .file_name_pattern("img_{index}_{hash}")
      .save_to_dir(images_dir.to_str().unwrap())
      .build();

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    // 测试配置和基本流程
    match result {
      Ok(call_result) => {
        assert_eq!(call_result.is_error, Some(false));
        assertions::assert_dir_exists(&images_dir);
      }
      Err(e) => panic!("不应该失败: {:?}", e),
    }
  }

  /// 集成测试：保存为新文件
  #[tokio::test]
  async fn integration_save_as_new_file() {
    let file_manager = TestFileManager::new();
    let original_file = file_manager.create_md_file("original.md", test_data::DOC_WITHOUT_IMAGES);
    let new_file_path = file_manager.temp_dir.path().join("processed.md");
    let assets_dir = file_manager.assets_dir();

    let config = ImageLocalizationConfigBuilder::new(original_file.to_str().unwrap())
      .save_to_dir(assets_dir.to_str().unwrap())
      .new_file_path(Some(new_file_path.to_str().unwrap().to_string()))
      .build();

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));

    // 验证原文件未被修改
    let original_content = std::fs::read_to_string(&original_file).unwrap();
    assert_eq!(original_content, test_data::DOC_WITHOUT_IMAGES);

    // 验证新文件被创建
    assert!(new_file_path.exists());
    let new_content = std::fs::read_to_string(&new_file_path).unwrap();
    assert_eq!(new_content, test_data::DOC_WITHOUT_IMAGES);

    // 验证目录被创建
    assertions::assert_dir_exists(&assets_dir);
  }

  /// 集成测试：大文档处理性能
  #[tokio::test]
  async fn integration_large_document_performance() {
    // 生成一个较大的文档（无图片）
    let mut large_content = String::new();
    for i in 1..=100 {
      large_content.push_str(&format!("# 第{}章\n\n", i));
      for j in 1..=5 {
        large_content.push_str(&format!("## {}.{} 小节\n\n", i, j));
        large_content.push_str("这是一些内容文本。\n\n");
      }
    }

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("large.md", &large_content);
    let assets_dir = file_manager.assets_dir();

    let start_time = std::time::Instant::now();

    let config =
      ImageLocalizationConfigBuilder::new(md_file.to_str().unwrap()).save_to_dir(assets_dir.to_str().unwrap()).build();

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    let duration = start_time.elapsed();

    // 验证处理时间合理（应该很快，因为没有图片）
    assert!(duration.as_secs() < 5, "处理时间过长: {:?}", duration);

    // 验证结果
    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));
    assertions::assert_dir_exists(&assets_dir);

    // 验证文档内容未被修改（因为没有图片）
    let final_content = std::fs::read_to_string(&md_file).unwrap();
    assert_eq!(final_content, large_content);
  }

  /// 集成测试：错误恢复和容错性
  #[tokio::test]
  async fn integration_error_recovery() {
    let file_manager = TestFileManager::new();

    // 测试各种错误情况
    let test_cases = vec![
      // 空文件路径
      ("", "应该处理空路径"),
      // 无效路径字符
      ("/invalid\0path/file.md", "应该处理无效路径字符"),
      // 权限问题路径（在某些系统上）
      ("/root/restricted.md", "应该处理权限问题"),
    ];

    for (invalid_path, description) in test_cases {
      let config = ImageLocalizationConfigBuilder::new(invalid_path)
        .save_to_dir(file_manager.assets_dir().to_str().unwrap())
        .build();

      let result = MarkdownToolsImpl::localize_images_impl(config).await;

      // 应该优雅地处理错误，不应该 panic
      match result {
        Ok(call_result) => {
          // 如果返回成功，应该标记为错误
          assert_eq!(call_result.is_error, Some(true), "{}", description);
        }
        Err(_) => {
          // 直接返回错误也是可接受的
          assert!(true, "{}", description);
        }
      }
    }
  }

  /// 集成测试：并发处理多个文件
  #[tokio::test]
  async fn integration_concurrent_processing() {
    let file_manager = TestFileManager::new();

    // 创建多个测试文件
    let mut tasks = Vec::new();

    for i in 1..=5 {
      let md_file = file_manager.create_md_file(&format!("concurrent_{}.md", i), test_data::DOC_WITHOUT_IMAGES);
      let assets_dir = file_manager.temp_dir.path().join(format!("assets_{}", i));

      let config = ImageLocalizationConfigBuilder::new(md_file.to_str().unwrap())
        .save_to_dir(assets_dir.to_str().unwrap())
        .build();

      // 创建异步任务
      let task = tokio::spawn(async move { MarkdownToolsImpl::localize_images_impl(config).await });

      tasks.push((task, assets_dir));
    }

    // 等待所有任务完成
    for (task, assets_dir) in tasks {
      let result = task.await.expect("Task should complete");

      assert!(result.is_ok());
      let call_result = result.unwrap();
      assert_eq!(call_result.is_error, Some(false));
      assertions::assert_dir_exists(&assets_dir);
    }
  }
}

#[cfg(feature = "mock")]
mod mock_integration_tests {
  use super::*;
  use rstest::rstest;

  #[rstest::fixture]
  async fn mock_server() -> MockHttpServer {
    let server = MockHttpServer::new().await;
    server.mock_basic_images().await;
    server
  }

  /// 集成测试：使用 Mock 服务器的完整工作流
  #[rstest]
  #[tokio::test]
  async fn integration_mock_server_workflow(#[future] mock_server: MockHttpServer) {
    let server = mock_server.await;
    let host = server.url();
    let content = test_data::doc_with_images(&host);

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("mock_test.md", &content);
    let assets_dir = file_manager.assets_dir();

    let config =
      ImageLocalizationConfigBuilder::new(md_file.to_str().unwrap()).save_to_dir(assets_dir.to_str().unwrap()).build();

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    // 验证基本流程
    match result {
      Ok(call_result) => {
        assert_eq!(call_result.is_error, Some(false));
        assertions::assert_dir_exists(&assets_dir);

        // 验证 Markdown 文件中的 URL 已更新
        let final_content = std::fs::read_to_string(&md_file).unwrap();
        assert!(final_content.contains("assets/"));
        assert!(!final_content.contains(&host)); // 不应该再包含原始 URL
      }
      Err(e) => panic!("Mock 测试不应该失败: {:?}", e),
    }
  }

  /// 集成测试：解析器与图片本地化的协同工作
  #[rstest]
  #[tokio::test]
  async fn integration_parser_with_localization(#[future] mock_server: MockHttpServer) {
    let server = mock_server.await;
    let host = server.url();
    let content = format!(
      r#"# 解析器集成测试

## Markdown 图片

![Markdown图片]({0}/image1.webp "Markdown标题")

## HTML 图片

<img src="{0}/image1.webp" alt="HTML图片" width="100" class="responsive">

## 混合内容

这里有文字和 ![内联图片]({0}/image1.png) 在同一行。

<p>HTML段落中的 <img src="{0}/image1.jpg" alt="段落图片"> 图片。</p>
"#,
      host
    );

    let file_manager = TestFileManager::new();
    let md_file = file_manager.create_md_file("parser_integration.md", &content);
    let assets_dir = file_manager.assets_dir();

    // 首先测试解析器能正确识别图片
    let parser = mcp_markdown_tools::parser::MarkdownParser::new().unwrap();
    let mst = parser.parse(&content).unwrap();

    // 验证解析器找到了图片
    let mut image_count = 0;
    mst.walk(&mut |node| {
      if node.is_image() {
        image_count += 1;
      }
    });

    // 应该找到一些图片节点
    assert!(image_count > 0);

    // 然后测试本地化流程
    let config =
      ImageLocalizationConfigBuilder::new(md_file.to_str().unwrap()).save_to_dir(assets_dir.to_str().unwrap()).build();

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    // 测试基本流程
    match result {
      Ok(call_result) => {
        assert_eq!(call_result.is_error, Some(false));
        assertions::assert_dir_exists(&assets_dir);
      }
      Err(e) => panic!("解析器集成测试不应该失败: {:?}", e),
    }
  }
}
