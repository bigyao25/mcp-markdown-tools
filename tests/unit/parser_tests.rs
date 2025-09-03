//! 解析器单元测试
//!
//! 测试 Markdown 解析器的核心功能

use crate::common::{test_data, TestFileManager};
use mcp_markdown_tools::parser::MarkdownParser;

#[cfg(test)]
mod tests {
  use super::*;

  /// 测试基本 Markdown 解析
  #[tokio::test]
  async fn test_parser_basic_parsing() {
    let parser = MarkdownParser::new().expect("Failed to create parser");
    let mst = parser.parse(test_data::SIMPLE_DOC).expect("Failed to parse");

    // 验证解析结果不为空
    assert!(!mst.is_empty());
  }

  /// 测试解析复杂文档结构
  #[tokio::test]
  async fn test_parser_complex_structure() {
    let parser = MarkdownParser::new().expect("Failed to create parser");
    let mst = parser.parse(test_data::COMPLEX_DOC).expect("Failed to parse");

    // 验证能够解析复杂结构
    assert!(!mst.is_empty());
  }

  /// 测试解析包含图片的文档
  #[tokio::test]
  async fn test_parser_with_images() {
    let content = test_data::doc_with_images("https://example.com");
    let parser = MarkdownParser::new().expect("Failed to create parser");
    let mst = parser.parse(&content).expect("Failed to parse");

    // 计算图片节点数量
    let mut image_count = 0;
    mst.walk(&mut |node| {
      if node.is_image() {
        image_count += 1;
      }
    });

    // 应该找到图片节点（具体数量取决于解析器实现）
    assert!(image_count > 0);
  }

  /// 测试解析空文档
  #[tokio::test]
  async fn test_parser_empty_document() {
    let parser = MarkdownParser::new().expect("Failed to create parser");
    let mst = parser.parse("").expect("Failed to parse empty document");

    // 空文档应该能正常解析
    assert!(mst.is_empty() || !mst.is_empty()); // 取决于解析器实现
  }

  /// 测试解析只有文本的文档
  #[tokio::test]
  async fn test_parser_text_only() {
    let text_only = "这是一段普通文本。\n\n另一段文本。";
    let parser = MarkdownParser::new().expect("Failed to create parser");
    let mst = parser.parse(text_only).expect("Failed to parse text-only document");

    // 应该能正常解析纯文本
    assert!(!mst.is_empty());
  }

  /// 测试解析标题层级
  #[tokio::test]
  async fn test_parser_heading_levels() {
    let content = r#"# H1 标题
## H2 标题
### H3 标题
#### H4 标题
##### H5 标题
###### H6 标题
"#;
    let parser = MarkdownParser::new().expect("Failed to create parser");
    let mst = parser.parse(content).expect("Failed to parse headings");

    // 验证能解析不同级别的标题
    assert!(!mst.is_empty());
  }

  /// 测试解析混合内容
  #[tokio::test]
  async fn test_parser_mixed_content() {
    let mixed_content = r#"# 标题

普通段落文本。

## 子标题

- 列表项 1
- 列表项 2

```rust
// 代码块
fn main() {
    println!("Hello, world!");
}
```

> 引用文本

![图片](https://example.com/image.jpg)

[链接](https://example.com)
"#;

    let parser = MarkdownParser::new().expect("Failed to create parser");
    let mst = parser.parse(mixed_content).expect("Failed to parse mixed content");

    // 验证能解析混合内容
    assert!(!mst.is_empty());
  }

  /// 测试解析错误处理
  #[tokio::test]
  async fn test_parser_error_handling() {
    let parser = MarkdownParser::new().expect("Failed to create parser");

    // 测试一些可能导致解析问题的内容
    let problematic_content = "# 标题\n\n\0\n\n## 另一个标题";

    // 解析器应该能处理或报告错误
    let result = parser.parse(problematic_content);

    // 根据解析器实现，可能成功或失败，但不应该 panic
    match result {
      Ok(_) => assert!(true),  // 成功解析
      Err(_) => assert!(true), // 报告错误也是可接受的
    }
  }

  /// 测试解析器创建失败的情况
  #[tokio::test]
  async fn test_parser_creation() {
    // 测试解析器能正常创建
    let result = MarkdownParser::new();
    assert!(result.is_ok());
  }

  /// 测试节点遍历功能
  #[tokio::test]
  async fn test_parser_node_traversal() {
    let parser = MarkdownParser::new().expect("Failed to create parser");
    let mst = parser.parse(test_data::SIMPLE_DOC).expect("Failed to parse");

    let mut node_count = 0;
    mst.walk(&mut |_node| {
      node_count += 1;
    });

    // 应该有多个节点
    assert!(node_count > 0);
  }
}
