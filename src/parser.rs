//! Markdown 解析器
//!
//! 将 Markdown 文本解析为 MST (Markdown Structured Tree) 结构

use crate::mst::{MSTNode, NodeType};
use regex::Regex;

/// Markdown 解析器
pub struct MarkdownParser {
  header_regex: Regex,
}

impl MarkdownParser {
  /// 创建新的解析器
  pub fn new() -> Result<Self, String> {
    let header_regex = Regex::new(r"^(#{1,6})\s+(.*)$").map_err(|e| format!("正则表达式错误: {}", e))?;

    Ok(Self { header_regex })
  }

  /// 解析 Markdown 文本为 MST
  pub fn parse(&self, content: &str) -> Result<MSTNode, String> {
    let mut root = MSTNode::new_root();
    let mut header_stack: Vec<(usize, usize)> = Vec::new(); // (level, node_index)

    for (line_number, line) in content.lines().enumerate() {
      let line_number = line_number + 1; // 从1开始计数

      if let Some(captures) = self.header_regex.captures(line) {
        let hashes = captures.get(1).unwrap().as_str();
        let title = captures.get(2).unwrap().as_str();
        let level = hashes.len();

        // 清理标题中的编号
        let clean_title = self.remove_numbering_from_title(title);

        let header_node = MSTNode::new_header(level, title.to_string(), clean_title, line.to_string(), line_number);

        // 找到合适的父节点
        self.insert_header_node(&mut root, &mut header_stack, header_node, level);
      } else {
        // 非标题行，作为内容节点
        let content_node = MSTNode::new_content(line.to_string(), line_number);

        // 将内容添加到最近的标题节点下，如果没有标题则添加到根节点
        if let Some((_, parent_index)) = header_stack.last() {
          self.add_content_to_node(&mut root, *parent_index, content_node);
        } else {
          root.add_child(content_node);
        }
      }
    }

    Ok(root)
  }

  /// 插入标题节点到合适的位置
  fn insert_header_node(
    &self,
    root: &mut MSTNode,
    header_stack: &mut Vec<(usize, usize)>,
    header_node: MSTNode,
    level: usize,
  ) {
    // 移除比当前级别高或相等的标题
    header_stack.retain(|(stack_level, _)| *stack_level < level);

    let node_index = if let Some((_, parent_index)) = header_stack.last() {
      // 添加到父标题下
      self.add_child_to_node(root, *parent_index, header_node)
    } else {
      // 添加到根节点下
      root.add_child(header_node);
      root.children.len() - 1
    };

    // 将新标题添加到栈中
    header_stack.push((level, self.calculate_node_index(root, &header_stack, node_index)));
  }

  /// 计算节点在树中的全局索引
  fn calculate_node_index(&self, _root: &MSTNode, header_stack: &[(usize, usize)], local_index: usize) -> usize {
    // 简化实现：使用路径来标识节点
    // 在实际实现中，可能需要更复杂的索引策略
    if header_stack.is_empty() {
      local_index
    } else {
      // 这里简化处理，实际应该计算完整路径
      local_index
    }
  }

  /// 向指定节点添加子节点
  fn add_child_to_node(&self, root: &mut MSTNode, _parent_index: usize, child: MSTNode) -> usize {
    // 简化实现：直接添加到最后一个标题节点
    // 在实际实现中，需要根据 parent_index 找到正确的父节点
    if let Some(last_header) = self.find_last_header_mut(root) {
      last_header.add_child(child);
      last_header.children.len() - 1
    } else {
      root.add_child(child);
      root.children.len() - 1
    }
  }

  /// 向指定节点添加内容
  fn add_content_to_node(&self, root: &mut MSTNode, _parent_index: usize, content: MSTNode) {
    // 简化实现：添加到最后一个标题节点
    if let Some(last_header) = self.find_last_header_mut(root) {
      last_header.add_child(content);
    } else {
      root.add_child(content);
    }
  }

  /// 找到最后一个标题节点（可变引用）
  fn find_last_header_mut<'a>(&self, node: &'a mut MSTNode) -> Option<&'a mut MSTNode> {
    // 从后往前查找最后一个标题节点
    for child in node.children.iter_mut().rev() {
      if child.is_header() {
        return Some(child);
      }
      if let Some(header) = self.find_last_header_mut(child) {
        return Some(header);
      }
    }
    None
  }

  /// 从标题中移除编号
  fn remove_numbering_from_title(&self, title: &str) -> String {
    let title = title.trim();

    // 定义各种编号模式的正则表达式
    let patterns = vec![
      // 阿拉伯数字编号：1. 1.1. 1.1.1. 等
      r"^\d+(\.\d+)*\.?\s*",
      // 中文数字编号：一、 二、 三、 一、一、 等
      r"^[一二三四五六七八九十百千万]+、(\s*[一二三四五六七八九十百千万]+、)*\s*",
      // 混合编号：一、1. 二、1.1. 等（先匹配中文部分）
      r"^[一二三四五六七八九十百千万]+、\s*\d+(\.\d+)*\.?\s*",
    ];

    let mut cleaned = title.to_string();

    for pattern in patterns {
      if let Ok(regex) = Regex::new(pattern) {
        cleaned = regex.replace(&cleaned, "").to_string();
      }
    }

    // 清理多余的空格
    cleaned.trim().to_string()
  }
}

impl Default for MarkdownParser {
  fn default() -> Self {
    Self::new().expect("Failed to create MarkdownParser")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parser_creation() {
    let parser = MarkdownParser::new();
    assert!(parser.is_ok());
  }

  #[test]
  fn test_parse_simple_headers() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"# 标题1
## 子标题1
### 子子标题1
## 子标题2
# 标题2"#;

    let mst = parser.parse(content).unwrap();
    let headers = mst.get_headers();

    assert_eq!(headers.len(), 5);
    assert_eq!(headers[0].header_level(), Some(1));
    assert_eq!(headers[0].clean_title.as_ref().unwrap(), "标题1");
    assert_eq!(headers[1].header_level(), Some(2));
    assert_eq!(headers[1].clean_title.as_ref().unwrap(), "子标题1");
  }

  #[test]
  fn test_parse_with_content() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"# 标题1

这是一段内容。

## 子标题1

更多内容。"#;

    let mst = parser.parse(content).unwrap();
    let headers = mst.get_headers();

    assert_eq!(headers.len(), 2);

    // 检查是否有内容节点
    let mut content_count = 0;
    mst.walk(&mut |node| {
      if node.is_content() {
        content_count += 1;
      }
    });

    assert!(content_count > 0);
  }

  #[test]
  fn test_remove_numbering_from_title() {
    let parser = MarkdownParser::new().unwrap();

    assert_eq!(parser.remove_numbering_from_title("1. 标题"), "标题");
    assert_eq!(parser.remove_numbering_from_title("1.1. 子标题"), "子标题");
    assert_eq!(parser.remove_numbering_from_title("一、标题"), "标题");
    assert_eq!(parser.remove_numbering_from_title("一、一、子标题"), "子标题");
    assert_eq!(parser.remove_numbering_from_title("标题"), "标题");
  }

  #[test]
  fn test_parse_empty_content() {
    let parser = MarkdownParser::new().unwrap();
    let mst = parser.parse("").unwrap();

    assert!(matches!(mst.node_type, NodeType::Root));
    assert_eq!(mst.children.len(), 0);
  }

  #[test]
  fn test_parse_no_headers() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"这是一段普通文本。
没有任何标题。

只是普通的段落。"#;

    let mst = parser.parse(content).unwrap();
    let headers = mst.get_headers();

    assert_eq!(headers.len(), 0);
    assert!(mst.children.len() > 0); // 应该有内容节点
  }
}
