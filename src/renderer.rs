//! Markdown 渲染器
//!
//! 将 MST 结构渲染回 Markdown 文本

use crate::mst::{MSTNode, NodeType};

/// Markdown 渲染器
pub struct MarkdownRenderer;

impl MarkdownRenderer {
  /// 创建新的渲染器
  pub fn new() -> Self {
    Self
  }

  /// 将 MST 渲染为 Markdown 文本
  pub fn render(&self, mst: &MSTNode) -> String {
    let mut result = Vec::new();
    self.render_node(mst, &mut result);
    result.join("\n")
  }

  /// 渲染单个节点
  fn render_node(&self, node: &MSTNode, result: &mut Vec<String>) {
    match &node.node_type {
      NodeType::Root => {
        // 根节点不输出内容，只处理子节点
        for child in &node.children {
          self.render_node(child, result);
        }
      }
      NodeType::Header(level) => {
        let hashes = "#".repeat(*level);
        let empty_title = String::new();
        let title = node.clean_title.as_ref().unwrap_or(&empty_title);

        // 如果有编号信息，添加编号
        let numbered_title = if let Some(numbering) = &node.numbering {
          format!("{}{}", numbering.formatted, title)
        } else {
          title.clone()
        };

        result.push(format!("{} {}", hashes, numbered_title));

        // 处理子节点
        for child in &node.children {
          self.render_node(child, result);
        }
      }
      NodeType::Content(content) => {
        result.push(content.clone());
      }
    }
  }

  /// 渲染为带编号的 Markdown（用于生成章节编号功能）
  pub fn render_with_numbering(&self, mst: &MSTNode) -> String {
    self.render(mst)
  }

  /// 渲染为无编号的 Markdown（用于清除章节编号功能）
  pub fn render_without_numbering(&self, mst: &MSTNode) -> String {
    let mut result = Vec::new();
    self.render_node_without_numbering(mst, &mut result);
    result.join("\n")
  }

  /// 渲染单个节点（不包含编号）
  fn render_node_without_numbering(&self, node: &MSTNode, result: &mut Vec<String>) {
    match &node.node_type {
      NodeType::Root => {
        // 根节点不输出内容，只处理子节点
        for child in &node.children {
          self.render_node_without_numbering(child, result);
        }
      }
      NodeType::Header(level) => {
        let hashes = "#".repeat(*level);
        let empty_title = String::new();
        let title = node.clean_title.as_ref().unwrap_or(&empty_title);

        result.push(format!("{} {}", hashes, title));

        // 处理子节点
        for child in &node.children {
          self.render_node_without_numbering(child, result);
        }
      }
      NodeType::Content(content) => {
        result.push(content.clone());
      }
    }
  }
}

impl Default for MarkdownRenderer {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::mst::{MSTNode, NumberingInfo};

  fn create_test_mst_with_numbering() -> MSTNode {
    let mut root = MSTNode::new_root();

    let mut h1 = MSTNode::new_header(1, "标题1".to_string(), "标题1".to_string(), "# 标题1".to_string(), 1);
    h1.numbering = Some(NumberingInfo { path: vec![1], formatted: "1. ".to_string() });

    let mut h2 = MSTNode::new_header(2, "子标题1".to_string(), "子标题1".to_string(), "## 子标题1".to_string(), 2);
    h2.numbering = Some(NumberingInfo { path: vec![1, 1], formatted: "1.1. ".to_string() });

    let content = MSTNode::new_content("这是一段内容。".to_string(), 3);

    h2.add_child(content);
    h1.add_child(h2);
    root.add_child(h1);

    root
  }

  fn create_test_mst_without_numbering() -> MSTNode {
    let mut root = MSTNode::new_root();

    let mut h1 = MSTNode::new_header(1, "标题1".to_string(), "标题1".to_string(), "# 标题1".to_string(), 1);
    let mut h2 = MSTNode::new_header(2, "子标题1".to_string(), "子标题1".to_string(), "## 子标题1".to_string(), 2);
    let content = MSTNode::new_content("这是一段内容。".to_string(), 3);

    h2.add_child(content);
    h1.add_child(h2);
    root.add_child(h1);

    root
  }

  #[test]
  fn test_render_with_numbering() {
    let mst = create_test_mst_with_numbering();
    let renderer = MarkdownRenderer::new();
    let result = renderer.render(&mst);

    let expected = r#"# 1. 标题1
## 1.1. 子标题1
这是一段内容。"#;

    assert_eq!(result, expected);
  }

  #[test]
  fn test_render_without_numbering() {
    let mst = create_test_mst_without_numbering();
    let renderer = MarkdownRenderer::new();
    let result = renderer.render_without_numbering(&mst);

    let expected = r#"# 标题1
## 子标题1
这是一段内容。"#;

    assert_eq!(result, expected);
  }

  #[test]
  fn test_render_empty_mst() {
    let root = MSTNode::new_root();
    let renderer = MarkdownRenderer::new();
    let result = renderer.render(&root);

    assert_eq!(result, "");
  }

  #[test]
  fn test_render_content_only() {
    let mut root = MSTNode::new_root();
    let content1 = MSTNode::new_content("第一行内容".to_string(), 1);
    let content2 = MSTNode::new_content("第二行内容".to_string(), 2);

    root.add_child(content1);
    root.add_child(content2);

    let renderer = MarkdownRenderer::new();
    let result = renderer.render(&root);

    let expected = r#"第一行内容
第二行内容"#;

    assert_eq!(result, expected);
  }

  #[test]
  fn test_render_headers_only() {
    let mut root = MSTNode::new_root();

    let mut h1 = MSTNode::new_header(1, "标题1".to_string(), "标题1".to_string(), "# 标题1".to_string(), 1);
    h1.numbering = Some(NumberingInfo { path: vec![1], formatted: "一、".to_string() });

    let mut h2 = MSTNode::new_header(2, "子标题1".to_string(), "子标题1".to_string(), "## 子标题1".to_string(), 2);
    h2.numbering = Some(NumberingInfo { path: vec![1, 1], formatted: "一、一、".to_string() });

    h1.add_child(h2);
    root.add_child(h1);

    let renderer = MarkdownRenderer::new();
    let result = renderer.render(&root);

    let expected = r#"# 一、标题1
## 一、一、子标题1"#;

    assert_eq!(result, expected);
  }

  #[test]
  fn test_render_deep_nesting() {
    let mut root = MSTNode::new_root();

    let mut h1 = MSTNode::new_header(1, "标题1".to_string(), "标题1".to_string(), "# 标题1".to_string(), 1);
    h1.numbering = Some(NumberingInfo { path: vec![1], formatted: "1. ".to_string() });

    let mut h2 = MSTNode::new_header(2, "子标题1".to_string(), "子标题1".to_string(), "## 子标题1".to_string(), 2);
    h2.numbering = Some(NumberingInfo { path: vec![1, 1], formatted: "1.1. ".to_string() });

    let mut h3 =
      MSTNode::new_header(3, "子子标题1".to_string(), "子子标题1".to_string(), "### 子子标题1".to_string(), 3);
    h3.numbering = Some(NumberingInfo { path: vec![1, 1, 1], formatted: "1.1.1. ".to_string() });

    h2.add_child(h3);
    h1.add_child(h2);
    root.add_child(h1);

    let renderer = MarkdownRenderer::new();
    let result = renderer.render(&root);

    let expected = r#"# 1. 标题1
## 1.1. 子标题1
### 1.1.1. 子子标题1"#;

    assert_eq!(result, expected);
  }
}
