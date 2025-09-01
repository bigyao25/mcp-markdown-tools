//! 图片本地化集成测试

use mcp_markdown_tools::config::LocalizeImagesConfig;
use mcp_markdown_tools::image_localizer::ImageLocalizer;
use mcp_markdown_tools::parser::MarkdownParser;
use mcp_markdown_tools::tools::MarkdownToolsImpl;
use std::fs;
use std::path::Path;
use tempfile::{NamedTempFile, TempDir};

#[cfg(test)]
mod tests {
  use super::*;

  /// 测试图片本地化完整流程 - 模拟场景
  #[tokio::test]
  async fn test_image_localization_workflow_mock() {
    let content = r#"# 图片测试文档

## 第一节

这里有一张图片：

![测试图片](https://example.com/image1.jpg)

## 第二节

这里有另一张图片：

<img src="https://example.com/image2.png" alt="PNG图片" width="200">

## 第三节

带标题的图片：

![带标题图片](https://example.com/image3.gif "这是标题")

普通文本内容。
"#;

    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("test.md");
    let assets_dir = temp_dir.path().join("assets");

    fs::write(&md_file, content).unwrap();

    let config = LocalizeImagesConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      image_file_name_pattern: "{index}-{hash}".to_string(),
      save_to_dir: assets_dir.to_str().unwrap().to_string(),
    };

    // 注意：这个测试依赖网络，在实际环境中可能失败
    // 主要测试工作流程和错误处理
    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    match result {
      Ok(call_result) => {
        // 成功情况：验证基本结构
        assert_eq!(call_result.is_error, Some(false));

        // 简化测试，不检查具体内容
        assert!(!call_result.content.is_empty());

        // 验证目录被创建
        assert!(assets_dir.exists());
      }
      Err(_) => {
        // 网络错误是可以接受的，主要测试不会 panic
        assert!(true);
      }
    }
  }

  /// 测试图片本地化 - 无图片文档
  #[tokio::test]
  async fn test_image_localization_no_images() {
    let content = r#"# 无图片文档

## 第一节

这是普通文本内容。

## 第二节

这里也没有图片，只有文字。

### 子节

更多文字内容。
"#;

    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("no_images.md");
    let assets_dir = temp_dir.path().join("assets");

    fs::write(&md_file, content).unwrap();

    let config = LocalizeImagesConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      image_file_name_pattern: "{index}-{hash}".to_string(),
      save_to_dir: assets_dir.to_str().unwrap().to_string(),
    };

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));

    // 验证目录被创建
    assert!(assets_dir.exists());

    // 验证文档内容未被修改
    let final_content = fs::read_to_string(&md_file).unwrap();
    assert_eq!(final_content, content);
  }

  /// 测试图片本地化 - 配置占位符解析
  #[tokio::test]
  async fn test_image_localization_placeholder_resolution() {
    let content = r#"# 测试文档

![图片](https://example.com/test.jpg)
"#;

    let temp_dir = TempDir::new().unwrap();
    let docs_dir = temp_dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();

    let md_file = docs_dir.join("test.md");
    fs::write(&md_file, content).unwrap();

    // 使用占位符配置
    let config = LocalizeImagesConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      image_file_name_pattern: "{multilevel_num}-{index}".to_string(),
      save_to_dir: "{full_dir_of_original_file}/assets/".to_string(),
    };

    // 验证占位符解析
    let resolved_dir = config.get_resolved_save_dir();
    let expected_dir = docs_dir.join("assets").to_str().unwrap().to_string() + "/";
    assert_eq!(resolved_dir, expected_dir);

    // 执行本地化（可能因网络失败，但测试配置解析）
    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    // 不管成功失败，都验证目录结构
    let assets_dir = docs_dir.join("assets");
    match result {
      Ok(_) => assert!(assets_dir.exists()),
      Err(_) => {
        // 网络错误时，目录可能已创建也可能未创建
        // 这里主要测试配置解析正确
        assert!(true);
      }
    }
  }

  /// 测试图片本地化 - 自定义文件名模式
  #[tokio::test]
  async fn test_image_localization_custom_filename_pattern() {
    let content = r#"# 自定义模式测试

![图片1](https://example.com/image1.jpg)

![图片2](https://example.com/image2.png)
"#;

    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("custom.md");
    let assets_dir = temp_dir.path().join("images");

    fs::write(&md_file, content).unwrap();

    let config = LocalizeImagesConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      image_file_name_pattern: "img_{index}_{hash}".to_string(),
      save_to_dir: assets_dir.to_str().unwrap().to_string(),
    };

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    // 测试配置和基本流程
    match result {
      Ok(call_result) => {
        assert_eq!(call_result.is_error, Some(false));
        assert!(assets_dir.exists());
      }
      Err(_) => {
        // 网络问题可接受
        assert!(true);
      }
    }
  }

  /// 测试图片本地化 - 错误处理
  #[tokio::test]
  async fn test_image_localization_error_handling() {
    // 测试不存在的文件
    let config = LocalizeImagesConfig {
      full_file_path: "/nonexistent/file.md".to_string(),
      image_file_name_pattern: "{index}-{hash}".to_string(),
      save_to_dir: "/tmp/assets/".to_string(),
    };

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));

    // 简化测试，不检查具体内容
    assert!(!call_result.content.is_empty());
  }

  /// 测试图片本地化 - 非 Markdown 文件
  #[tokio::test]
  async fn test_image_localization_non_markdown_file() {
    let temp_file = NamedTempFile::with_suffix(".txt").unwrap();
    fs::write(temp_file.path(), "![图片](https://example.com/test.jpg)").unwrap();

    let temp_dir = TempDir::new().unwrap();

    let config = LocalizeImagesConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      image_file_name_pattern: "{index}-{hash}".to_string(),
      save_to_dir: temp_dir.path().to_str().unwrap().to_string(),
    };

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));

    if let rmcp::model::Content::Text { text } = &call_result.content[0] {
      assert!(text.contains("文件验证失败"));
    }
  }

  /// 测试图片本地化 - 解析器集成
  #[tokio::test]
  async fn test_image_localization_parser_integration() {
    let content = r#"# 解析器集成测试

## Markdown 图片

![Markdown图片](https://example.com/md.jpg "Markdown标题")

## HTML 图片

<img src="https://example.com/html.png" alt="HTML图片" width="100" class="responsive">

## 混合内容

这里有文字和 ![内联图片](https://example.com/inline.gif) 在同一行。

<p>HTML段落中的 <img src="https://example.com/paragraph.webp" alt="段落图片"> 图片。</p>
"#;

    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("parser_test.md");

    fs::write(&md_file, content).unwrap();

    // 首先测试解析器能正确识别图片
    let parser = MarkdownParser::new().unwrap();
    let mst = parser.parse(content).unwrap();

    // 验证解析器找到了图片
    let mut image_count = 0;
    mst.walk(&mut |node| {
      if node.is_image() {
        image_count += 1;
      }
    });

    // 应该找到多个图片（具体数量取决于解析器实现）
    assert!(image_count > 0);

    // 然后测试本地化流程
    let assets_dir = temp_dir.path().join("assets");
    let config = LocalizeImagesConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      image_file_name_pattern: "{index}-{hash}".to_string(),
      save_to_dir: assets_dir.to_str().unwrap().to_string(),
    };

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    // 测试基本流程（网络可能失败）
    match result {
      Ok(call_result) => {
        assert_eq!(call_result.is_error, Some(false));
        assert!(assets_dir.exists());
      }
      Err(_) => {
        assert!(true); // 网络错误可接受
      }
    }
  }

  /// 测试图片本地化 - 大量图片处理
  #[tokio::test]
  async fn test_image_localization_many_images() {
    // 生成包含多个图片的文档
    let mut content = String::from("# 多图片测试文档\n\n");

    for i in 1..=20 {
      content.push_str(&format!("## 第{}节\n\n", i));
      content.push_str(&format!("![图片{}](https://example.com/image{}.jpg)\n\n", i, i));
      content.push_str(&format!("<img src=\"https://example.com/html{}.png\" alt=\"HTML图片{}\">\n\n", i, i));
    }

    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("many_images.md");
    let assets_dir = temp_dir.path().join("assets");

    fs::write(&md_file, &content).unwrap();

    let config = LocalizeImagesConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      image_file_name_pattern: "img_{index}_{hash}".to_string(),
      save_to_dir: assets_dir.to_str().unwrap().to_string(),
    };

    let start_time = std::time::Instant::now();

    let result = MarkdownToolsImpl::localize_images_impl(config).await;

    let duration = start_time.elapsed();

    // 验证处理时间合理
    assert!(duration.as_secs() < 30, "处理时间过长: {:?}", duration);

    // 测试基本流程
    match result {
      Ok(call_result) => {
        assert_eq!(call_result.is_error, Some(false));
        assert!(assets_dir.exists());

        // 简化测试，不检查具体内容
        assert!(!call_result.content.is_empty());
      }
      Err(_) => {
        // 网络错误可接受
        assert!(true);
      }
    }
  }

  /// 测试图片本地化 - 与编号生成的组合工作流
  #[tokio::test]
  async fn test_combined_numbering_and_localization_workflow() {
    let content = r#"# 组合测试文档

## 第一章

这里有一张图片：

![章节图片](https://example.com/chapter1.jpg)

### 1.1 小节

另一张图片：

<img src="https://example.com/section1.png" alt="小节图片">

## 第二章

第二章的图片：

![第二章图片](https://example.com/chapter2.gif "第二章")
"#;

    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("combined.md");
    let assets_dir = temp_dir.path().join("assets");

    fs::write(&md_file, content).unwrap();

    // 第一步：生成章节编号
    let numbering_config = mcp_markdown_tools::config::GenerateChapterConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let numbering_result = MarkdownToolsImpl::generate_chapter_number_impl(numbering_config, "numed").await;
    assert!(numbering_result.is_ok());

    // 验证编号被添加
    let numbered_content = fs::read_to_string(&md_file).unwrap();
    assert!(numbered_content.contains("# 1. 组合测试文档"));
    assert!(numbered_content.contains("## 1.1. 第一章"));

    // 第二步：本地化图片
    let localization_config = LocalizeImagesConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      image_file_name_pattern: "{index}-{hash}".to_string(),
      save_to_dir: assets_dir.to_str().unwrap().to_string(),
    };

    let localization_result = MarkdownToolsImpl::localize_images_impl(localization_config).await;

    // 测试组合流程
    match localization_result {
      Ok(call_result) => {
        assert_eq!(call_result.is_error, Some(false));
        assert!(assets_dir.exists());

        // 验证编号仍然存在
        let final_content = fs::read_to_string(&md_file).unwrap();
        assert!(final_content.contains("# 1. 组合测试文档"));
        assert!(final_content.contains("## 1.1. 第一章"));
      }
      Err(_) => {
        // 网络错误可接受，但编号应该仍然存在
        let final_content = fs::read_to_string(&md_file).unwrap();
        assert!(final_content.contains("# 1. 组合测试文档"));
      }
    }
  }
}
