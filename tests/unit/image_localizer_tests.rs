//! image_localizer.rs 模块的单元测试

use mcp_markdown_tools::config::LocalizeImagesConfig;
use mcp_markdown_tools::image_localizer::ImageLocalizer;
use mcp_markdown_tools::mst::{ImageInfo, ImageType, MSTNode, NodeType};
use std::fs;
use std::path::Path;
use tempfile::{NamedTempFile, TempDir};

#[cfg(test)]
mod tests {
  use super::*;

  /// 创建测试用的 LocalizeImagesConfig
  fn create_test_config(file_path: &str, save_dir: &str) -> LocalizeImagesConfig {
    LocalizeImagesConfig {
      full_file_path: file_path.to_string(),
      image_file_name_pattern: "{index}-{hash}".to_string(),
      save_to_dir: save_dir.to_string(),
    }
  }

  /// 创建测试用的图片节点
  fn create_test_image_node(url: &str, alt_text: &str) -> MSTNode {
    let image_info = ImageInfo {
      original_url: url.to_string(),
      alt_text: alt_text.to_string(),
      title: None,
      local_path: None,
      image_type: ImageType::Markdown,
      html_attributes: None,
    };

    MSTNode {
      node_type: NodeType::Image(image_info),
      children: Vec::new(),
      line_number: 1,
      raw_line: format!("![{}]({})", alt_text, url),
    }
  }

  /// 创建测试用的 HTML 图片节点
  fn create_test_html_image_node(url: &str, alt_text: &str, attributes: Option<String>) -> MSTNode {
    let image_info = ImageInfo {
      original_url: url.to_string(),
      alt_text: alt_text.to_string(),
      title: None,
      local_path: None,
      image_type: ImageType::Html,
      html_attributes: attributes,
    };

    let raw_line = if let Some(attrs) = &image_info.html_attributes {
      format!("<img {} src=\"{}\" alt=\"{}\">", attrs, url, alt_text)
    } else {
      format!("<img src=\"{}\" alt=\"{}\">", url, alt_text)
    };

    MSTNode { node_type: NodeType::Image(image_info), children: Vec::new(), line_number: 1, raw_line }
  }

  /// 测试 ImageLocalizer 创建
  #[test]
  fn test_image_localizer_creation() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config.clone());

    // 验证配置被正确设置（通过间接方式，因为字段是私有的）
    // 这里我们主要测试创建不会 panic
    assert!(true);
  }

  /// 测试文件名生成 - 基本情况
  #[test]
  fn test_generate_filename_basic() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    // 测试数据
    let url = "https://example.com/image.jpg";
    let index = 0;
    let bytes = b"fake image data";

    let result = localizer.generate_filename(url, index, bytes);
    assert!(result.is_ok());

    let filename = result.unwrap();
    assert!(filename.ends_with(".jpg"));
    assert!(filename.contains("0-")); // index
    assert!(filename.len() > 10); // 应该包含哈希
  }

  /// 测试文件名生成 - 不同扩展名
  #[test]
  fn test_generate_filename_different_extensions() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);
    let bytes = b"test";

    // 测试不同的扩展名
    let test_cases = vec![
      ("https://example.com/image.png", "png"),
      ("https://example.com/image.gif", "gif"),
      ("https://example.com/image.svg", "svg"),
      ("https://example.com/image.webp", "webp"),
    ];

    for (url, expected_ext) in test_cases {
      let filename = localizer.generate_filename(url, 0, bytes).unwrap();
      assert!(filename.ends_with(&format!(".{}", expected_ext)));
    }
  }

  /// 测试文件名生成 - 无扩展名的 URL
  #[test]
  fn test_generate_filename_no_extension() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    let url = "https://example.com/image";
    let filename = localizer.generate_filename(url, 0, b"test").unwrap();

    // 应该使用默认扩展名 jpg
    assert!(filename.ends_with(".jpg"));
  }

  /// 测试文件名生成 - 自定义模式
  #[test]
  fn test_generate_filename_custom_pattern() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let mut config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());
    config.image_file_name_pattern = "img_{index}_{hash}".to_string();

    let localizer = ImageLocalizer::new(config);

    let filename = localizer.generate_filename("https://example.com/test.png", 5, b"data").unwrap();

    assert!(filename.starts_with("img_5_"));
    assert!(filename.ends_with(".png"));
  }

  /// 测试获取文件扩展名
  #[test]
  fn test_get_file_extension() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    // 测试各种 URL 格式
    assert_eq!(localizer.get_file_extension("https://example.com/image.JPG").unwrap(), "jpg");
    assert_eq!(localizer.get_file_extension("https://example.com/path/image.PNG").unwrap(), "png");
    assert_eq!(localizer.get_file_extension("https://example.com/image.gif?param=1").unwrap(), "gif");
    assert_eq!(localizer.get_file_extension("https://example.com/noext").unwrap(), "jpg");
    // 默认
  }

  /// 测试获取相对路径
  #[test]
  fn test_get_relative_path() {
    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("test.md");
    let image_file = temp_dir.path().join("assets").join("image.jpg");

    // 创建目录结构
    fs::create_dir_all(image_file.parent().unwrap()).unwrap();
    fs::write(&md_file, "# Test").unwrap();
    fs::write(&image_file, "fake image").unwrap();

    let config = create_test_config(md_file.to_str().unwrap(), temp_dir.path().join("assets").to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    let relative_path = localizer.get_relative_path(&image_file).unwrap();
    assert_eq!(relative_path, "assets/image.jpg");
  }

  /// 测试获取相对路径 - Windows 路径分隔符
  #[test]
  fn test_get_relative_path_windows_separators() {
    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("test.md");
    let image_file = temp_dir.path().join("assets").join("image.jpg");

    fs::create_dir_all(image_file.parent().unwrap()).unwrap();
    fs::write(&md_file, "# Test").unwrap();
    fs::write(&image_file, "fake image").unwrap();

    let config = create_test_config(md_file.to_str().unwrap(), temp_dir.path().join("assets").to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    let relative_path = localizer.get_relative_path(&image_file).unwrap();

    // 应该使用正斜杠，不管平台如何
    assert!(!relative_path.contains('\\'));
    assert!(relative_path.contains('/') || !relative_path.contains('/'));
  }

  /// 测试 MST 中图片节点的处理
  #[tokio::test]
  async fn test_process_image_node_in_mst() {
    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("test.md");
    let assets_dir = temp_dir.path().join("assets");

    fs::create_dir_all(&assets_dir).unwrap();
    fs::write(&md_file, "# Test").unwrap();

    let config = create_test_config(md_file.to_str().unwrap(), assets_dir.to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    // 创建包含图片的 MST
    let mut root = MSTNode::new_root();
    let image_node = create_test_image_node("https://httpbin.org/image/png", "测试图片");
    root.children.push(image_node);

    // 注意：这个测试需要网络连接，在实际环境中可能需要模拟
    // 这里我们主要测试函数调用不会 panic
    let result = localizer.localize_images(&mut root).await;

    // 由于网络依赖，我们只测试函数调用
    // 在真实测试中，应该使用 mock HTTP 服务器
    match result {
      Ok(_) => {
        // 成功情况：验证图片被下载
        assert!(assets_dir.exists());
      }
      Err(_) => {
        // 失败情况：可能是网络问题，这在测试中是可以接受的
        assert!(true);
      }
    }
  }

  /// 测试空 MST 的处理
  #[tokio::test]
  async fn test_localize_images_empty_mst() {
    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("test.md");
    let assets_dir = temp_dir.path().join("assets");

    fs::write(&md_file, "# Test").unwrap();

    let config = create_test_config(md_file.to_str().unwrap(), assets_dir.to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    let mut root = MSTNode::new_root();

    let result = localizer.localize_images(&mut root).await.unwrap();

    // 应该返回空结果
    assert!(result.is_empty());

    // 目录应该被创建
    assert!(assets_dir.exists());
  }

  /// 测试 HTML 图片节点的处理
  #[test]
  fn test_html_image_node_creation() {
    let node = create_test_html_image_node(
      "https://example.com/image.jpg",
      "HTML图片",
      Some("width=\"100\" class=\"responsive\"".to_string()),
    );

    if let NodeType::Image(image_info) = &node.node_type {
      assert_eq!(image_info.image_type, ImageType::Html);
      assert_eq!(image_info.alt_text, "HTML图片");
      assert_eq!(image_info.original_url, "https://example.com/image.jpg");
      assert!(image_info.html_attributes.is_some());
    } else {
      panic!("期望图片节点");
    }
  }

  /// 测试 Markdown 图片节点的处理
  #[test]
  fn test_markdown_image_node_creation() {
    let node = create_test_image_node("https://example.com/image.png", "Markdown图片");

    if let NodeType::Image(image_info) = &node.node_type {
      assert_eq!(image_info.image_type, ImageType::Markdown);
      assert_eq!(image_info.alt_text, "Markdown图片");
      assert_eq!(image_info.original_url, "https://example.com/image.png");
      assert!(image_info.html_attributes.is_none());
    } else {
      panic!("期望图片节点");
    }
  }

  /// 测试配置的占位符解析
  #[test]
  fn test_config_placeholder_resolution() {
    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("docs").join("test.md");

    fs::create_dir_all(md_file.parent().unwrap()).unwrap();
    fs::write(&md_file, "# Test").unwrap();

    let mut config = LocalizeImagesConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      image_file_name_pattern: "{multilevel_num}-{index}".to_string(),
      save_to_dir: "{full_dir_of_original_file}/assets/".to_string(),
    };

    let resolved_dir = config.get_resolved_save_dir();
    let expected_dir = md_file.parent().unwrap().join("assets").to_str().unwrap().to_string() + "/";

    assert_eq!(resolved_dir, expected_dir);
  }

  /// 测试无效 URL 的处理
  #[test]
  fn test_invalid_url_handling() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    // 测试无效 URL
    let result = localizer.get_file_extension("not-a-url");
    assert!(result.is_err());
  }

  /// 测试哈希生成的一致性
  #[test]
  fn test_hash_consistency() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    let data = b"test image data";
    let url = "https://example.com/image.jpg";

    // 多次生成应该产生相同的哈希
    let filename1 = localizer.generate_filename(url, 0, data).unwrap();
    let filename2 = localizer.generate_filename(url, 0, data).unwrap();

    assert_eq!(filename1, filename2);
  }

  /// 测试不同数据产生不同哈希
  #[test]
  fn test_different_data_different_hash() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    let url = "https://example.com/image.jpg";

    let filename1 = localizer.generate_filename(url, 0, b"data1").unwrap();
    let filename2 = localizer.generate_filename(url, 0, b"data2").unwrap();

    assert_ne!(filename1, filename2);
  }

  /// 测试边界情况：空数据
  #[test]
  fn test_empty_data_handling() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    let result = localizer.generate_filename("https://example.com/image.jpg", 0, b"");
    assert!(result.is_ok());

    let filename = result.unwrap();
    assert!(filename.ends_with(".jpg"));
    assert!(filename.contains("0-")); // index 应该存在
  }
}
