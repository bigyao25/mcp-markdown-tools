//! 图片本地化模块
//!
//! 负责下载远程图片并保存到本地

use crate::config::LocalizeImagesConfig;
use crate::mst::{ImageInfo, MSTNode};
use reqwest;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use url::Url;

/// 图片本地化器
pub struct ImageLocalizer {
  config: LocalizeImagesConfig,
  client: reqwest::Client,
}

impl ImageLocalizer {
  /// 创建新的图片本地化器
  pub fn new(config: LocalizeImagesConfig) -> Self {
    let client = reqwest::Client::builder()
      .timeout(Duration::from_secs(10)) // 总超时时间
      .build()
      .unwrap();
    Self { config, client }
  }

  /// 本地化 MST 中的所有图片
  pub async fn localize_images(&self, mst: &mut MSTNode) -> Result<Vec<String>, String> {
    let mut results = Vec::new();

    // 确保保存目录存在，使用处理占位符后的路径
    let save_dir = PathBuf::from(self.config.get_resolved_save_dir());
    fs::create_dir_all(&save_dir).map_err(|e| format!("创建目录失败: {}", e))?;

    // 递归处理所有图片节点
    let mut index = 0;
    self.process_images_recursive(mst, &mut index, &save_dir, &mut results).await?;

    Ok(results)
  }

  /// 递归处理图片节点和包含图片的内容节点
  fn process_images_recursive<'a>(
    &'a self,
    node: &'a mut MSTNode,
    index: &'a mut usize,
    save_dir: &'a Path,
    results: &'a mut Vec<String>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(async move {
      // 处理图片节点
      if let Some(image_info) = node.get_image_info_mut() {
        match self.download_and_save_image(image_info, *index, save_dir).await {
          Ok(local_path) => {
            results.push(format!("✅ 成功下载: {} -> {}", image_info.original_url, local_path));
            image_info.local_path = Some(local_path);
          }
          Err(e) => {
            results.push(format!("❌ 下载失败: {} - {}", image_info.original_url, e));
          }
        }
        *index += 1;
      }

      // 处理包含图片的内容节点
      if node.is_content() {
        if let crate::mst::NodeType::Content(content) = &mut node.node_type {
          let updated_content = self.process_inline_images_in_content(content, index, save_dir, results).await?;
          *content = updated_content;
        }
      }

      for child in &mut node.children {
        self.process_images_recursive(child, index, save_dir, results).await?;
      }

      Ok(())
    })
  }

  /// 下载并保存图片
  async fn download_and_save_image(
    &self,
    image_info: &ImageInfo,
    index: usize,
    save_dir: &Path,
  ) -> Result<String, String> {
    // 下载图片
    let response = self.client.get(&image_info.original_url).send().await.map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
      return Err(format!("HTTP 错误: {}", response.status()));
    }

    let content_type =
      response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    let bytes = response.bytes().await.map_err(|e| format!("读取响应失败: {}", e))?;

    // 生成文件名
    let filename = self.generate_filename(&image_info.original_url, index, content_type.as_deref(), &bytes)?;
    let file_path = save_dir.join(&filename);

    // 保存文件
    fs::write(&file_path, &bytes).map_err(|e| format!("保存文件失败: {}", e))?;

    // 返回相对路径
    let relative_path = self.get_relative_path(&file_path)?;
    Ok(relative_path)
  }

  /// 生成文件名
  fn generate_filename(
    &self,
    url: &str,
    index: usize,
    content_type: Option<&str>,
    bytes: &[u8],
  ) -> Result<String, String> {
    // 获取文件扩展名
    let extension = self.get_file_extension(url, content_type)?;

    // 生成哈希
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = format!("{:x}", hasher.finalize())[..6].to_string();

    // 替换模式中的占位符
    let mut filename = self.config.image_file_name_pattern.clone();
    filename = filename.replace("{index}", &index.to_string());
    filename = filename.replace("{hash}", &hash);

    // TODO: 实现 multilevel_num 的替换
    // NOTE: multilevel_num为图片所处的标题行层级路径，比如 1.2.1.3.
    filename = filename.replace("{multilevel_num}", &format!("{}", index + 1));

    Ok(format!("{}.{}", filename, extension))
  }

  /// 获取文件扩展名
  fn get_file_extension(&self, url: &str, content_type: Option<&str>) -> Result<String, String> {
    let parsed_url = Url::parse(url).map_err(|e| format!("解析 URL 失败: {}", e))?;
    let path = parsed_url.path();

    if let Some(extension) = Path::new(path).extension() {
      if let Some(ext_str) = extension.to_str() {
        return Ok(ext_str.to_lowercase());
      }
    }

    Ok(content_type.unwrap_or("image/jpg").replace("image/", "").to_string())
  }

  /// 获取相对于 Markdown 文件的相对路径
  fn get_relative_path(&self, file_path: &Path) -> Result<String, String> {
    let md_file_path = Path::new(&self.config.full_file_path);
    let md_dir = md_file_path.parent().ok_or("无法获取 Markdown 文件目录")?;

    let relative_path = file_path.strip_prefix(md_dir).map_err(|_| "无法计算相对路径")?;

    relative_path.to_str().ok_or("路径包含无效字符".to_string()).map(|s| s.replace('\\', "/"))
    // 统一使用正斜杠
  }

  /// 处理内容中的行内图片（简化版本，复用解析器逻辑）
  async fn process_inline_images_in_content(
    &self,
    content: &str,
    index: &mut usize,
    save_dir: &Path,
    results: &mut Vec<String>,
  ) -> Result<String, String> {
    // 复用解析器的图片解析逻辑
    let parser = crate::parser::MarkdownParser::new().map_err(|e| format!("创建解析器失败: {}", e))?;
    let image_nodes = parser.parse_images_in_line(content, 0);

    // 如果没有图片，直接返回原内容
    if image_nodes.is_empty() {
      return Ok(content.to_string());
    }

    let mut updated_content = content.to_string();

    // 处理每个图片节点
    for image_node in image_nodes {
      if let Some(image_info) = image_node.get_image_info() {
        // 使用通用的图片处理函数
        match self.process_single_image_for_content(image_info, *index, save_dir, results).await {
          Ok((original_text, replacement_text)) => {
            updated_content = updated_content.replace(&original_text, &replacement_text);
            *index += 1;
          }
          Err(e) => {
            results.push(format!("❌ 处理图片失败: {} - {}", image_info.original_url, e));
          }
        }
      }
    }

    Ok(updated_content)
  }

  /// 处理单个图片（通用函数）
  async fn process_single_image_for_content(
    &self,
    image_info: &ImageInfo,
    index: usize,
    save_dir: &Path,
    results: &mut Vec<String>,
  ) -> Result<(String, String), String> {
    // 下载并保存图片
    let local_path = self.download_and_save_image(image_info, index, save_dir).await?;

    results.push(format!("✅ 成功下载: {} -> {}", image_info.original_url, local_path));

    // 构建原始文本和替换文本
    let (original_text, replacement_text) = match image_info.image_type {
      crate::mst::ImageType::Markdown => {
        // 构建原始 Markdown 图片引用
        let title_part = image_info.title.as_ref().map(|t| format!(" \"{}\"", t)).unwrap_or_default();
        let original = format!("![{}]({}{})", image_info.alt_text, image_info.original_url, title_part);

        // 构建替换后的 Markdown 图片引用
        let replacement = format!("![{}]({}{})", image_info.alt_text, local_path, title_part);

        (original, replacement)
      }
      crate::mst::ImageType::Html => {
        // 构建原始 HTML img 标签
        let mut original_tag = String::from("<img");
        if let Some(attrs) = &image_info.html_attributes {
          if !attrs.is_empty() {
            original_tag.push(' ');
            original_tag.push_str(attrs);
          }
        }
        original_tag.push_str(&format!(" src=\"{}\"", image_info.original_url));
        if !image_info.alt_text.is_empty() {
          let attrs_str = image_info.html_attributes.as_deref().unwrap_or("");
          if !attrs_str.contains("alt=") {
            original_tag.push_str(&format!(" alt=\"{}\"", image_info.alt_text));
          }
        }
        original_tag.push('>');

        // 构建替换后的 HTML img 标签
        let mut replacement_tag = String::from("<img");
        if let Some(attrs) = &image_info.html_attributes {
          if !attrs.is_empty() {
            replacement_tag.push(' ');
            replacement_tag.push_str(attrs);
          }
        }
        replacement_tag.push_str(&format!(" src=\"{}\"", local_path));
        if !image_info.alt_text.is_empty() {
          let attrs_str = image_info.html_attributes.as_deref().unwrap_or("");
          if !attrs_str.contains("alt=") {
            replacement_tag.push_str(&format!(" alt=\"{}\"", image_info.alt_text));
          }
        }
        replacement_tag.push('>');

        (original_tag, replacement_tag)
      }
    };

    Ok((original_text, replacement_text))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::mst::{ImageInfo, ImageType, MSTNode, NodeType};
  use std::fs;
  use tempfile::{NamedTempFile, TempDir};

  /// 创建测试用的 LocalizeImagesConfig
  fn create_test_config(file_path: &str, save_dir: &str) -> LocalizeImagesConfig {
    LocalizeImagesConfig {
      full_file_path: file_path.to_string(),
      image_file_name_pattern: "{index}-{hash}".to_string(),
      image_dir: save_dir.to_string(),
      new_full_file_path: None,
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
      title: None,

      raw: format!("![{}]({})", alt_text, url),
      line_number: 1,
      children: Vec::new(),
      numbering: None,
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

    let raw = if let Some(attrs) = &image_info.html_attributes {
      format!("<img {} src=\"{}\" alt=\"{}\">", attrs, url, alt_text)
    } else {
      format!("<img src=\"{}\" alt=\"{}\">", url, alt_text)
    };

    MSTNode {
      node_type: NodeType::Image(image_info),
      title: None,

      raw,
      line_number: 1,
      children: Vec::new(),
      numbering: None,
    }
  }

  /// 测试 ImageLocalizer 创建
  #[test]
  fn test_image_localizer_creation() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let _localizer = ImageLocalizer::new(config.clone());

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
    let url = "https://example.com/image.svg";
    let index = 0;
    let bytes = b"fake image data";

    let result = localizer.generate_filename(url, index, None, bytes);

    let re = regex::Regex::new(r"\d+-\w{6}.svg").unwrap();
    match result {
      Ok(file_name) => assert!(re.is_match(file_name.as_str()), "file_name ({}) is not match", file_name),
      Err(_) => assert!(false, "generate_filename failed"),
    }
  }

  /// 测试文件名生成 - 扩展名来自 URL
  #[test]
  fn test_generate_filename_extension_from_url() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);
    let bytes = b"test";

    // 测试不同的扩展名
    let test_cases = vec![
      ("https://example.com/image.png", Some("image/png"), "png"),
      ("https://example.com/image.gif", Some("image/gif"), "gif"),
      ("https://example.com/image.svg", Some("image/svg"), "svg"),
      ("https://example.com/image.webp", Some("image/webp"), "webp"),
    ];

    for (url, content_type, expected_ext) in test_cases {
      let filename = localizer.generate_filename(url, 0, content_type, bytes).unwrap();
      assert!(filename.ends_with(&format!(".{}", expected_ext)));
    }
  }

  /// 测试文件名生成 - 扩展名来自 content-type
  #[test]
  fn test_generate_filename_extension_from_content_type() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    let url = "https://example.com/image";
    let filename = localizer.generate_filename(url, 0, Some("image/svg"), b"test").unwrap();

    // 应该使用默认扩展名 jpg
    assert!(filename.ends_with(".svg"));
  }

  /// 测试文件名生成 - 默认扩展名
  #[test]
  fn test_generate_filename_no_extension() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    let url = "https://example.com/image";
    let filename = localizer.generate_filename(url, 0, None, b"test").unwrap();

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

    let filename = localizer.generate_filename("https://example.com/test.png", 5, Some("image/png"), b"data").unwrap();

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
    assert_eq!(localizer.get_file_extension("https://example.com/image.JPG", None).unwrap(), "jpg");
    assert_eq!(localizer.get_file_extension("https://example.com/path/image.PNG", None).unwrap(), "png");
    assert_eq!(localizer.get_file_extension("https://example.com/image.gif?param=1", None).unwrap(), "gif");
    assert_eq!(localizer.get_file_extension("https://example.com/noext", None).unwrap(), "jpg");
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
      assert!(false, "期望图片节点");
    }
  }

  /// 测试配置的占位符解析
  #[test]
  fn test_config_placeholder_resolution() {
    let temp_dir = TempDir::new().unwrap();
    let md_file = temp_dir.path().join("docs").join("test.md");

    fs::create_dir_all(md_file.parent().unwrap()).unwrap();
    fs::write(&md_file, "# Test").unwrap();

    let config = LocalizeImagesConfig {
      full_file_path: md_file.to_str().unwrap().to_string(),
      image_file_name_pattern: "{multilevel_num}-{index}".to_string(),
      image_dir: "{full_dir_of_original_file}/assets/".to_string(),
      new_full_file_path: None,
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
    let result = localizer.get_file_extension("not-a-url", None);
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
    let filename1 = localizer.generate_filename(url, 0, Some("image/jpg"), data).unwrap();
    let filename2 = localizer.generate_filename(url, 0, Some("image/jpg"), data).unwrap();

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

    let filename1 = localizer.generate_filename(url, 0, Some("image/jpg"), b"data1").unwrap();
    let filename2 = localizer.generate_filename(url, 0, Some("image/jpg"), b"data2").unwrap();

    assert_ne!(filename1, filename2);
  }

  /// 测试边界情况：空数据
  #[test]
  fn test_empty_data_handling() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_dir = TempDir::new().unwrap();

    let config = create_test_config(temp_file.path().to_str().unwrap(), temp_dir.path().to_str().unwrap());

    let localizer = ImageLocalizer::new(config);

    let result = localizer.generate_filename("https://example.com/image.jpg", 0, Some("image/jpg"), b"");

    let re = regex::Regex::new(r"\d+-\w{6}.jpg").unwrap();
    match result {
      Ok(file_name) => assert!(re.is_match(file_name.as_str()), "file_name ({}) is not match", file_name),
      Err(_) => assert!(false, "generate_filename failed"),
    }
  }
}
