//! 图片本地化模块
//!
//! 负责下载远程图片并保存到本地

use crate::config::LocalizeImagesConfig;
use crate::mst::{ImageInfo, MSTNode};
use reqwest;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

/// 图片本地化器
pub struct ImageLocalizer {
  config: LocalizeImagesConfig,
  client: reqwest::Client,
}

impl ImageLocalizer {
  /// 创建新的图片本地化器
  pub fn new(config: LocalizeImagesConfig) -> Self {
    let client = reqwest::Client::new();
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

    let bytes = response.bytes().await.map_err(|e| format!("读取响应失败: {}", e))?;

    // 生成文件名
    let filename = self.generate_filename(&image_info.original_url, index, &bytes)?;
    let file_path = save_dir.join(&filename);

    // 保存文件
    fs::write(&file_path, &bytes).map_err(|e| format!("保存文件失败: {}", e))?;

    // 返回相对路径
    let relative_path = self.get_relative_path(&file_path)?;
    Ok(relative_path)
  }

  /// 生成文件名
  fn generate_filename(&self, url: &str, index: usize, bytes: &[u8]) -> Result<String, String> {
    // 获取文件扩展名
    let extension = self.get_file_extension(url)?;

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
  fn get_file_extension(&self, url: &str) -> Result<String, String> {
    let parsed_url = Url::parse(url).map_err(|e| format!("解析 URL 失败: {}", e))?;
    let path = parsed_url.path();

    if let Some(extension) = Path::new(path).extension() {
      if let Some(ext_str) = extension.to_str() {
        return Ok(ext_str.to_lowercase());
      }
    }

    // 默认扩展名
    Ok("jpg".to_string())
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
