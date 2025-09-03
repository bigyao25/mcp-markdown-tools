//! MST (Markdown Structured Tree) - Markdown 文档结构化树
//!
//! 这个模块提供了一个树形结构来表示 Markdown 文档的层次结构，
//! 将解析、处理和渲染逻辑分离，提高代码的可维护性和扩展性。

use std::fmt;

/// Markdown 文档节点类型
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
  /// 根节点
  Root,
  /// 标题节点 (级别: 1-6)
  Header(usize),
  /// 文本内容节点
  Content(String),
  /// 图片节点
  Image(ImageInfo),
}

/// 图片类型
#[derive(Debug, Clone, PartialEq)]
pub enum ImageType {
  /// Markdown 格式：![alt](url "title")
  Markdown,
  /// HTML img 标签格式：<img src="url" ...>
  Html,
}

/// 图片信息
#[derive(Debug, Clone, PartialEq)]
pub struct ImageInfo {
  /// 图片类型
  pub image_type: ImageType,
  /// 图片的原始 URL
  pub original_url: String,
  /// 本地化后的路径（如果已本地化）
  pub local_path: Option<String>,
  /// 图片的 alt 文本
  pub alt_text: String,
  /// 图片的标题（可选）
  pub title: Option<String>,
  /// HTML 属性（仅对 HTML 类型有效）
  pub html_attributes: Option<String>,
}

/// MST 节点
#[derive(Debug, Clone)]
pub struct MSTNode {
  /// 节点类型
  pub node_type: NodeType,
  /// 标题文本（移除了编号，仅对 Header 节点有效）
  pub title: Option<String>,
  /// 原始行内容
  pub raw_line: String,
  /// 行号（从1开始）
  pub line_number: usize,
  /// 子节点
  pub children: Vec<MSTNode>,
  /// 编号信息（处理后填充）
  pub numbering: Option<NumberingInfo>,
}

/// 编号信息
#[derive(Debug, Clone)]
pub struct NumberingInfo {
  /// 编号路径 [1, 2, 1] 表示 1.2.1
  pub path: Vec<usize>,
  /// 格式化后的编号字符串
  pub formatted: String,
}

/// 编号配置
#[derive(Debug, Clone)]
pub struct NumberingConfig {
  /// 是否忽略一级标题
  pub ignore_h1: bool,
  /// 是否使用中文编号
  pub use_chinese_number: bool,
  /// 子级是否使用阿拉伯数字（仅当 use_chinese_number=true 时有效）
  pub use_arabic_number_for_sublevel: bool,
}

impl Default for NumberingConfig {
  fn default() -> Self {
    Self { ignore_h1: false, use_chinese_number: false, use_arabic_number_for_sublevel: true }
  }
}

impl MSTNode {
  /// 创建根节点
  pub fn new_root() -> Self {
    Self {
      node_type: NodeType::Root,
      title: None,
      raw_line: String::new(),
      line_number: 0,
      children: Vec::new(),
      numbering: None,
    }
  }

  /// 创建标题节点
  pub fn new_header(level: usize, title: String, raw_line: String, line_number: usize) -> Self {
    Self {
      node_type: NodeType::Header(level),
      title: Some(title),
      raw_line,
      line_number,
      children: Vec::new(),
      numbering: None,
    }
  }

  /// 创建内容节点
  pub fn new_content(content: String, line_number: usize) -> Self {
    Self {
      node_type: NodeType::Content(content.clone()),
      title: None,
      raw_line: content,
      line_number,
      children: Vec::new(),
      numbering: None,
    }
  }

  /// 创建图片节点
  pub fn new_image(image_info: ImageInfo, raw_line: String, line_number: usize) -> Self {
    Self {
      node_type: NodeType::Image(image_info),
      title: None,
      raw_line,
      line_number,
      children: Vec::new(),
      numbering: None,
    }
  }

  /// 获取标题级别
  pub fn header_level(&self) -> Option<usize> {
    match self.node_type {
      NodeType::Header(level) => Some(level),
      _ => None,
    }
  }

  /// 是否为标题节点
  pub fn is_header(&self) -> bool {
    matches!(self.node_type, NodeType::Header(_))
  }

  /// 是否为内容节点
  pub fn is_content(&self) -> bool {
    matches!(self.node_type, NodeType::Content(_))
  }

  /// 是否为图片节点
  pub fn is_image(&self) -> bool {
    matches!(self.node_type, NodeType::Image(_))
  }

  /// 获取图片信息（如果是图片节点）
  pub fn get_image_info(&self) -> Option<&ImageInfo> {
    match &self.node_type {
      NodeType::Image(info) => Some(info),
      _ => None,
    }
  }

  /// 获取图片信息（可变引用，如果是图片节点）
  pub fn get_image_info_mut(&mut self) -> Option<&mut ImageInfo> {
    match &mut self.node_type {
      NodeType::Image(info) => Some(info),
      _ => None,
    }
  }

  /// 添加子节点
  pub fn add_child(&mut self, child: MSTNode) {
    self.children.push(child);
  }

  /// 递归遍历所有节点
  pub fn walk<F>(&self, callback: &mut F)
  where
    F: FnMut(&MSTNode),
  {
    callback(self);
    for child in &self.children {
      child.walk(callback);
    }
  }

  /// 递归遍历所有节点（可变引用）
  pub fn walk_mut<F>(&mut self, callback: &mut F)
  where
    F: FnMut(&mut MSTNode),
  {
    callback(self);
    for child in &mut self.children {
      child.walk_mut(callback);
    }
  }

  /// 获取所有标题节点
  pub fn get_headers(&self) -> Vec<&MSTNode> {
    let mut headers = Vec::new();
    self.collect_headers(&mut headers);
    headers
  }

  fn collect_headers<'a>(&'a self, headers: &mut Vec<&'a MSTNode>) {
    if self.is_header() {
      headers.push(self);
    }
    for child in &self.children {
      child.collect_headers(headers);
    }
  }

  /// 获取所有标题节点（可变引用）- 简化版本，直接修改节点
  pub fn apply_to_headers<F>(&mut self, f: &mut F)
  where
    F: FnMut(&mut MSTNode),
  {
    if self.is_header() {
      f(self);
    }
    for child in &mut self.children {
      child.apply_to_headers(f);
    }
  }
}

impl fmt::Display for MSTNode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.fmt_with_indent(f, 0)
  }
}

impl MSTNode {
  fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
    let indent_str = "  ".repeat(indent);

    match &self.node_type {
      NodeType::Root => {
        writeln!(f, "{}Root", indent_str)?;
      }
      NodeType::Header(level) => {
        let empty_title = String::new();
        let title = self.title.as_ref().unwrap_or(&empty_title);
        let numbering = self.numbering.as_ref().map(|n| format!(" [{}]", n.formatted)).unwrap_or_default();
        writeln!(f, "{}H{}: {}{}", indent_str, level, title, numbering)?;
      }
      NodeType::Content(content) => {
        let preview = if content.len() > 50 { format!("{}...", &content[..47]) } else { content.clone() };
        writeln!(f, "{}Content: {}", indent_str, preview)?;
      }
      NodeType::Image(image_info) => {
        let url = image_info.local_path.as_ref().unwrap_or(&image_info.original_url);
        writeln!(f, "{}Image: {} (alt: {})", indent_str, url, image_info.alt_text)?;
      }
    }

    for child in &self.children {
      child.fmt_with_indent(f, indent + 1)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_mst_node_creation() {
    let root = MSTNode::new_root();
    assert!(matches!(root.node_type, NodeType::Root));
    assert_eq!(root.children.len(), 0);

    let header = MSTNode::new_header(1, "Title".to_string(), "# Title".to_string(), 1);
    assert_eq!(header.header_level(), Some(1));
    assert!(header.is_header());
    assert!(!header.is_content());

    let content = MSTNode::new_content("Some text".to_string(), 2);
    assert!(content.is_content());
    assert!(!content.is_header());
  }

  #[test]
  fn test_mst_tree_structure() {
    let mut root = MSTNode::new_root();
    let mut h1 = MSTNode::new_header(1, "Chapter 1".to_string(), "# Chapter 1".to_string(), 1);
    let h2 = MSTNode::new_header(2, "Section 1.1".to_string(), "## Section 1.1".to_string(), 2);

    h1.add_child(h2);
    root.add_child(h1);

    assert_eq!(root.children.len(), 1);
    assert_eq!(root.children[0].children.len(), 1);
  }

  #[test]
  fn test_get_headers() {
    let mut root = MSTNode::new_root();
    let mut h1 = MSTNode::new_header(1, "Chapter 1".to_string(), "# Chapter 1".to_string(), 1);
    let h2 = MSTNode::new_header(2, "Section 1.1".to_string(), "## Section 1.1".to_string(), 2);
    let content = MSTNode::new_content("Some text".to_string(), 3);

    h1.add_child(h2);
    h1.add_child(content);
    root.add_child(h1);

    let headers = root.get_headers();
    assert_eq!(headers.len(), 2); // h1 and h2
    assert_eq!(headers[0].header_level(), Some(1));
    assert_eq!(headers[1].header_level(), Some(2));
  }

  #[test]
  fn test_image_node_creation() {
    let image_info = ImageInfo {
      image_type: ImageType::Markdown,
      original_url: "https://example.com/image.png".to_string(),
      local_path: None,
      alt_text: "Test image".to_string(),
      title: Some("Image title".to_string()),
      html_attributes: None,
    };

    let image_node = MSTNode::new_image(
      image_info.clone(),
      "![Test image](https://example.com/image.png \"Image title\")".to_string(),
      5,
    );

    assert!(image_node.is_image());
    assert!(!image_node.is_header());
    assert!(!image_node.is_content());
    assert_eq!(image_node.line_number, 5);

    let retrieved_info = image_node.get_image_info().unwrap();
    assert_eq!(retrieved_info.original_url, "https://example.com/image.png");
    assert_eq!(retrieved_info.alt_text, "Test image");
    assert_eq!(retrieved_info.title, Some("Image title".to_string()));
    assert_eq!(retrieved_info.local_path, None);
  }

  #[test]
  fn test_image_info_mutation() {
    let image_info = ImageInfo {
      image_type: ImageType::Markdown,
      original_url: "https://example.com/image.png".to_string(),
      local_path: None,
      alt_text: "Test image".to_string(),
      title: None,
      html_attributes: None,
    };

    let mut image_node = MSTNode::new_image(image_info, "![Test image](https://example.com/image.png)".to_string(), 1);

    // 测试可变引用
    let info_mut = image_node.get_image_info_mut().unwrap();
    info_mut.local_path = Some("./assets/image.png".to_string());
    info_mut.title = Some("Updated title".to_string());

    let info = image_node.get_image_info().unwrap();
    assert_eq!(info.local_path, Some("./assets/image.png".to_string()));
    assert_eq!(info.title, Some("Updated title".to_string()));
  }

  #[test]
  fn test_walk_functionality() {
    let mut root = MSTNode::new_root();
    let mut h1 = MSTNode::new_header(1, "Chapter 1".to_string(), "# Chapter 1".to_string(), 1);
    let h2 = MSTNode::new_header(2, "Section 1.1".to_string(), "## Section 1.1".to_string(), 2);
    let content = MSTNode::new_content("Some text".to_string(), 3);

    h1.add_child(h2);
    h1.add_child(content);
    root.add_child(h1);

    let mut visited_nodes = Vec::new();
    root.walk(&mut |node| {
      visited_nodes.push(node.line_number);
    });

    assert_eq!(visited_nodes, vec![0, 1, 2, 3]); // root, h1, h2, content
  }

  #[test]
  fn test_walk_mut_functionality() {
    let mut root = MSTNode::new_root();
    let mut h1 = MSTNode::new_header(1, "Chapter 1".to_string(), "# Chapter 1".to_string(), 1);
    let h2 = MSTNode::new_header(2, "Section 1.1".to_string(), "## Section 1.1".to_string(), 2);

    h1.add_child(h2);
    root.add_child(h1);

    // 使用 walk_mut 修改所有节点的行号
    root.walk_mut(&mut |node| {
      node.line_number += 10;
    });

    assert_eq!(root.line_number, 10);
    assert_eq!(root.children[0].line_number, 11);
    assert_eq!(root.children[0].children[0].line_number, 12);
  }

  #[test]
  fn test_apply_to_headers() {
    let mut root = MSTNode::new_root();
    let mut h1 = MSTNode::new_header(1, "Chapter 1".to_string(), "# Chapter 1".to_string(), 1);
    let h2 = MSTNode::new_header(2, "Section 1.1".to_string(), "## Section 1.1".to_string(), 2);
    let content = MSTNode::new_content("Some text".to_string(), 3);

    h1.add_child(h2);
    h1.add_child(content);
    root.add_child(h1);

    // 为所有标题添加编号信息
    root.apply_to_headers(&mut |node| {
      if let Some(level) = node.header_level() {
        node.numbering = Some(NumberingInfo { path: vec![level], formatted: format!("{}.", level) });
      }
    });

    let headers = root.get_headers();
    assert!(headers[0].numbering.is_some());
    assert!(headers[1].numbering.is_some());
    assert_eq!(headers[0].numbering.as_ref().unwrap().formatted, "1.");
    assert_eq!(headers[1].numbering.as_ref().unwrap().formatted, "2.");
  }

  #[test]
  fn test_numbering_config_default() {
    let config = NumberingConfig::default();
    assert!(!config.ignore_h1);
    assert!(!config.use_chinese_number);
    assert!(config.use_arabic_number_for_sublevel);
  }

  #[test]
  fn test_numbering_info() {
    let numbering = NumberingInfo { path: vec![1, 2, 3], formatted: "1.2.3".to_string() };

    assert_eq!(numbering.path, vec![1, 2, 3]);
    assert_eq!(numbering.formatted, "1.2.3");
  }

  #[test]
  fn test_node_type_equality() {
    assert_eq!(NodeType::Root, NodeType::Root);
    assert_eq!(NodeType::Header(1), NodeType::Header(1));
    assert_ne!(NodeType::Header(1), NodeType::Header(2));

    let content1 = NodeType::Content("test".to_string());
    let content2 = NodeType::Content("test".to_string());
    let content3 = NodeType::Content("different".to_string());
    assert_eq!(content1, content2);
    assert_ne!(content1, content3);

    let image_info1 = ImageInfo {
      image_type: ImageType::Markdown,
      original_url: "url1".to_string(),
      local_path: None,
      alt_text: "alt1".to_string(),
      title: None,
      html_attributes: None,
    };
    let image_info2 = ImageInfo {
      image_type: ImageType::Markdown,
      original_url: "url1".to_string(),
      local_path: None,
      alt_text: "alt1".to_string(),
      title: None,
      html_attributes: None,
    };
    assert_eq!(NodeType::Image(image_info1), NodeType::Image(image_info2));
  }

  #[test]
  fn test_display_formatting() {
    let mut root = MSTNode::new_root();
    let mut h1 = MSTNode::new_header(1, "Chapter 1".to_string(), "# Chapter 1".to_string(), 1);

    // 添加编号信息
    h1.numbering = Some(NumberingInfo { path: vec![1], formatted: "1.".to_string() });

    let content = MSTNode::new_content(
      "This is a very long content that should be truncated when displayed in the tree view".to_string(),
      2,
    );

    let image_info = ImageInfo {
      image_type: ImageType::Markdown,
      original_url: "https://example.com/image.png".to_string(),
      local_path: Some("./assets/image.png".to_string()),
      alt_text: "Test image".to_string(),
      title: None,
      html_attributes: None,
    };
    let image_node = MSTNode::new_image(image_info, "![Test image](./assets/image.png)".to_string(), 3);

    h1.add_child(content);
    h1.add_child(image_node);
    root.add_child(h1);

    let display_output = format!("{}", root);

    // 验证输出包含预期的格式
    assert!(display_output.contains("Root"));
    assert!(display_output.contains("H1: Chapter 1 [1.]"));
    // 内容 "This is a very long content that should be truncated when displayed in the tree view"
    // 前47个字符是 "This is a very long content that should be trun"
    assert!(display_output.contains("Content: This is a very long content that should be trun..."));
    assert!(display_output.contains("Image: ./assets/image.png (alt: Test image)"));
  }

  #[test]
  fn test_empty_tree() {
    let root = MSTNode::new_root();
    assert_eq!(root.children.len(), 0);
    assert_eq!(root.get_headers().len(), 0);

    let mut visited = 0;
    root.walk(&mut |_| visited += 1);
    assert_eq!(visited, 1); // 只有根节点
  }

  #[test]
  fn test_mixed_content_tree() {
    let mut root = MSTNode::new_root();

    // 添加各种类型的节点
    let content1 = MSTNode::new_content("Introduction".to_string(), 1);
    let h1 = MSTNode::new_header(1, "Chapter 1".to_string(), "# Chapter 1".to_string(), 2);
    let content2 = MSTNode::new_content("Chapter content".to_string(), 3);

    let image_info = ImageInfo {
      image_type: ImageType::Markdown,
      original_url: "https://example.com/diagram.png".to_string(),
      local_path: None,
      alt_text: "Diagram".to_string(),
      title: Some("Architecture Diagram".to_string()),
      html_attributes: None,
    };
    let image = MSTNode::new_image(image_info, "![Diagram](https://example.com/diagram.png)".to_string(), 4);

    root.add_child(content1);
    root.add_child(h1);
    root.add_child(content2);
    root.add_child(image);

    assert_eq!(root.children.len(), 4);
    assert_eq!(root.get_headers().len(), 1);

    // 验证节点类型
    assert!(root.children[0].is_content());
    assert!(root.children[1].is_header());
    assert!(root.children[2].is_content());
    assert!(root.children[3].is_image());
  }
}
