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
}

/// MST 节点
#[derive(Debug, Clone)]
pub struct MSTNode {
  /// 节点类型
  pub node_type: NodeType,
  /// 原始标题文本（仅对 Header 节点有效）
  pub title: Option<String>,
  /// 清理后的标题文本（移除了编号）
  pub clean_title: Option<String>,
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
      clean_title: None,
      raw_line: String::new(),
      line_number: 0,
      children: Vec::new(),
      numbering: None,
    }
  }

  /// 创建标题节点
  pub fn new_header(level: usize, title: String, clean_title: String, raw_line: String, line_number: usize) -> Self {
    Self {
      node_type: NodeType::Header(level),
      title: Some(title),
      clean_title: Some(clean_title),
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
      clean_title: None,
      raw_line: content,
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
        let title = self.clean_title.as_ref().unwrap_or(&empty_title);
        let numbering = self.numbering.as_ref().map(|n| format!(" [{}]", n.formatted)).unwrap_or_default();
        writeln!(f, "{}H{}: {}{}", indent_str, level, title, numbering)?;
      }
      NodeType::Content(content) => {
        let preview = if content.len() > 50 { format!("{}...", &content[..47]) } else { content.clone() };
        writeln!(f, "{}Content: {}", indent_str, preview)?;
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

    let header = MSTNode::new_header(1, "Title".to_string(), "Title".to_string(), "# Title".to_string(), 1);
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
    let mut h1 = MSTNode::new_header(1, "Chapter 1".to_string(), "Chapter 1".to_string(), "# Chapter 1".to_string(), 1);
    let h2 =
      MSTNode::new_header(2, "Section 1.1".to_string(), "Section 1.1".to_string(), "## Section 1.1".to_string(), 2);

    h1.add_child(h2);
    root.add_child(h1);

    assert_eq!(root.children.len(), 1);
    assert_eq!(root.children[0].children.len(), 1);
  }

  #[test]
  fn test_get_headers() {
    let mut root = MSTNode::new_root();
    let mut h1 = MSTNode::new_header(1, "Chapter 1".to_string(), "Chapter 1".to_string(), "# Chapter 1".to_string(), 1);
    let h2 =
      MSTNode::new_header(2, "Section 1.1".to_string(), "Section 1.1".to_string(), "## Section 1.1".to_string(), 2);
    let content = MSTNode::new_content("Some text".to_string(), 3);

    h1.add_child(h2);
    h1.add_child(content);
    root.add_child(h1);

    let headers = root.get_headers();
    assert_eq!(headers.len(), 2); // h1 and h2
    assert_eq!(headers[0].header_level(), Some(1));
    assert_eq!(headers[1].header_level(), Some(2));
  }
}
