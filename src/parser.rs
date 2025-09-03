//! Markdown 解析器
//!
//! 将 Markdown 文本解析为 MST (Markdown Structured Tree) 结构

use crate::mst::{ImageInfo, MSTNode, NodeType};
use regex::Regex;

/// Markdown 解析器
pub struct MarkdownParser {
  header_regex: Regex,
  image_regex: Regex,
  html_img_regex: Regex,
}

impl MarkdownParser {
  /// 创建新的解析器
  pub fn new() -> Result<Self, String> {
    let header_regex = Regex::new(r"^(#{1,6})\s+(.*)$").map_err(|e| format!("正则表达式错误: {}", e))?;
    let image_regex =
      Regex::new(r#"!\[([^\]]*)\]\(([^)]+?)(?:\s+"([^"]*)")?\)"#).map_err(|e| format!("图片正则表达式错误: {}", e))?;
    let html_img_regex = Regex::new(r#"<img\s*([^>]*?)src\s*=\s*["']([^"']+)["']([^>]*?)/?>"#)
      .map_err(|e| format!("HTML img 正则表达式错误: {}", e))?;

    Ok(Self { header_regex, image_regex, html_img_regex })
  }

  /// 解析 Markdown 文本为 MST
  pub fn parse(&self, content: &str) -> Result<MSTNode, String> {
    let mut root = MSTNode::new_root();
    let mut header_stack: Vec<(usize, usize)> = Vec::new(); // (level, node_index)

    for (line_number, line) in content.lines().enumerate() {
      let line_number = line_number + 1; // 从1开始计数

      // 标题行
      if let Some(captures) = self.header_regex.captures(line) {
        let hashes = captures.get(1).unwrap().as_str();
        let title = captures.get(2).unwrap().as_str();
        let level = hashes.len();

        // 清理标题中的编号
        let title = self.remove_numbering_from_title(title);

        let header_node = MSTNode::new_header(level, title, line.to_string(), line_number);

        // 找到合适的父节点
        self.insert_header_node(&mut root, &mut header_stack, header_node, level);
        continue;
      }

      // 检查是否包含图片（可能是行内图片或独立图片行）
      let image_nodes = self.parse_images_in_line(line, line_number);
      if !image_nodes.is_empty() {
        // 如果整行只有一个图片且没有其他内容，作为独立图片节点
        if image_nodes.len() == 1 && line.trim() == image_nodes[0].raw_line.trim() {
          if let Some((_, parent_index)) = header_stack.last() {
            self.add_content_to_node(&mut root, *parent_index, image_nodes.into_iter().next().unwrap());
          } else {
            root.add_child(image_nodes.into_iter().next().unwrap());
          }
          continue;
        } else {
          // 行内图片：只创建一个内容节点，包含图片信息但不创建单独的图片节点
          // 图片信息会在图片本地化过程中被处理和替换
          let content_node = MSTNode::new_content(line.to_string(), line_number);

          // 将图片信息附加到内容节点，以便后续处理
          // 这里我们需要一种方式来标记这个内容节点包含图片
          // 暂时先只创建内容节点，图片信息在本地化时通过正则表达式重新解析

          if let Some((_, parent_index)) = header_stack.last() {
            self.add_content_to_node(&mut root, *parent_index, content_node);
          } else {
            root.add_child(content_node);
          }
          continue;
        }
      }

      // 普通内容节点
      let content_node = MSTNode::new_content(line.to_string(), line_number);

      // 将内容添加到最近的标题节点下，如果没有标题则添加到根节点
      if let Some((_, parent_index)) = header_stack.last() {
        self.add_content_to_node(&mut root, *parent_index, content_node);
      } else {
        root.add_child(content_node);
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

  /// 解析一行中的所有图片（支持行内图片和多个图片）
  pub fn parse_images_in_line(&self, line: &str, line_number: usize) -> Vec<MSTNode> {
    let mut images = Vec::new();

    // 解析 Markdown 格式的图片
    for captures in self.image_regex.captures_iter(line) {
      let alt_text = captures.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
      let url = captures.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
      let title = captures.get(3).map(|m| m.as_str().to_string());

      // 检查是否是远程图片（http/https）
      if url.starts_with("http://") || url.starts_with("https://") {
        let image_info = ImageInfo {
          image_type: crate::mst::ImageType::Markdown,
          original_url: url,
          local_path: None,
          alt_text,
          title,
          html_attributes: None,
        };

        let full_match = captures.get(0).unwrap().as_str();
        images.push(MSTNode::new_image(image_info, full_match.to_string(), line_number));
      }
    }

    // 解析 HTML img 标签
    for captures in self.html_img_regex.captures_iter(line) {
      let before_attrs = captures.get(1).map(|m| m.as_str()).unwrap_or("").trim();
      let url = captures.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
      let after_attrs = captures.get(3).map(|m| m.as_str()).unwrap_or("").trim();

      // 检查是否是远程图片（http/https）
      if url.starts_with("http://") || url.starts_with("https://") {
        // 合并所有属性
        let mut all_attrs = Vec::new();
        if !before_attrs.is_empty() {
          all_attrs.push(before_attrs);
        }
        if !after_attrs.is_empty() {
          all_attrs.push(after_attrs);
        }
        let html_attributes = if all_attrs.is_empty() { None } else { Some(all_attrs.join(" ")) };

        // 尝试从属性中提取 alt 文本
        let alt_text = self.extract_alt_from_attributes(&format!("{} {}", before_attrs, after_attrs));

        let image_info = ImageInfo {
          image_type: crate::mst::ImageType::Html,
          original_url: url,
          local_path: None,
          alt_text,
          title: None, // HTML img 标签通常不使用 title 属性作为图片标题
          html_attributes,
        };

        let full_match = captures.get(0).unwrap().as_str();
        images.push(MSTNode::new_image(image_info, full_match.to_string(), line_number));
      }
    }

    images
  }

  /// 从 HTML 属性中提取 alt 文本
  fn extract_alt_from_attributes(&self, attrs: &str) -> String {
    // 简单的 alt 属性提取
    if let Ok(alt_regex) = Regex::new(r#"alt\s*=\s*["']([^"']*)["']"#) {
      if let Some(captures) = alt_regex.captures(attrs) {
        return captures.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
      }
    }
    String::new()
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
  use crate::renderer::MarkdownRenderer;

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
    assert_eq!(headers[0].title.as_ref().unwrap(), "标题1");
    assert_eq!(headers[1].header_level(), Some(2));
    assert_eq!(headers[1].title.as_ref().unwrap(), "子标题1");
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

  #[test]
  fn test_parse_images() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"# 图片测试

![Alt text](https://example.com/image.png)

![带标题的图片](https://example.com/image2.jpg "图片标题")

## 本地图片

![本地图片](./local/image.png)

普通文本内容。"#;

    let mst = parser.parse(content).unwrap();

    // 检查图片节点数量
    let mut image_count = 0;
    mst.walk(&mut |node| {
      if node.is_image() {
        image_count += 1;
      }
    });

    assert_eq!(image_count, 2); // 只有远程图片会被解析为图片节点

    // 验证第一个图片节点
    let mut found_first_image = false;
    mst.walk(&mut |node| {
      if let Some(image_info) = node.get_image_info() {
        if !found_first_image {
          assert_eq!(image_info.original_url, "https://example.com/image.png");
          assert_eq!(image_info.alt_text, "Alt text");
          assert_eq!(image_info.title, None);
          assert_eq!(image_info.local_path, None);
          found_first_image = true;
        }
      }
    });
    assert!(found_first_image);
  }

  #[test]
  fn test_parse_complex_numbering() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"# 1. 第一章
## 1.1. 第一节
### 1.1.1. 第一小节
## 1.2. 第二节
# 2. 第二章
## 2.1. 第一节"#;

    let mst = parser.parse(content).unwrap();
    let headers = mst.get_headers();

    assert_eq!(headers.len(), 6);

    // 验证编号被正确移除
    assert_eq!(headers[0].title.as_ref().unwrap(), "第一章");
    assert_eq!(headers[1].title.as_ref().unwrap(), "第一节");
    assert_eq!(headers[2].title.as_ref().unwrap(), "第一小节");
    assert_eq!(headers[3].title.as_ref().unwrap(), "第二节");
    assert_eq!(headers[4].title.as_ref().unwrap(), "第二章");
    assert_eq!(headers[5].title.as_ref().unwrap(), "第一节");
  }

  #[test]
  fn test_parse_chinese_numbering() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"# 一、第一章
## 一、一、第一节
### 一、一、一、第一小节
## 一、二、第二节
# 二、第二章"#;

    let mst = parser.parse(content).unwrap();
    let headers = mst.get_headers();

    assert_eq!(headers.len(), 5);

    // 验证中文编号被正确移除
    assert_eq!(headers[0].title.as_ref().unwrap(), "第一章");
    assert_eq!(headers[1].title.as_ref().unwrap(), "第一节");
    assert_eq!(headers[2].title.as_ref().unwrap(), "第一小节");
    assert_eq!(headers[3].title.as_ref().unwrap(), "第二节");
    assert_eq!(headers[4].title.as_ref().unwrap(), "第二章");
  }

  #[test]
  fn test_parse_mixed_content_types() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"前言内容

# 第一章

章节介绍

![图片](https://example.com/diagram.png "架构图")

## 第一节

节内容

更多内容

### 子节

子节内容

# 第二章

第二章内容"#;

    let mst = parser.parse(content).unwrap();

    // 统计各种节点类型
    let mut header_count = 0;
    let mut content_count = 0;
    let mut image_count = 0;

    mst.walk(&mut |node| match &node.node_type {
      NodeType::Header(_) => header_count += 1,
      NodeType::Content(_) => content_count += 1,
      NodeType::Image(_) => image_count += 1,
      _ => {}
    });

    assert_eq!(header_count, 4); // 4个标题
    assert!(content_count > 0); // 有内容节点
    assert_eq!(image_count, 1); // 1个图片节点
  }

  #[test]
  fn test_header_hierarchy() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"# H1
## H2
### H3
## H2-2
# H1-2"#;

    let mst = parser.parse(content).unwrap();
    let headers = mst.get_headers();

    assert_eq!(headers.len(), 5);
    assert_eq!(headers[0].header_level(), Some(1));
    assert_eq!(headers[1].header_level(), Some(2));
    assert_eq!(headers[2].header_level(), Some(3));
    assert_eq!(headers[3].header_level(), Some(2));
    assert_eq!(headers[4].header_level(), Some(1));
  }

  #[test]
  fn test_remove_numbering_edge_cases() {
    let parser = MarkdownParser::new().unwrap();

    // 测试边界情况
    assert_eq!(parser.remove_numbering_from_title(""), "");
    assert_eq!(parser.remove_numbering_from_title("   "), "");
    assert_eq!(parser.remove_numbering_from_title("1."), "");
    assert_eq!(parser.remove_numbering_from_title("一、"), "");
    assert_eq!(parser.remove_numbering_from_title("1. "), "");
    assert_eq!(parser.remove_numbering_from_title("一、 "), "");
    assert_eq!(parser.remove_numbering_from_title("标题 1."), "标题 1.");
    assert_eq!(parser.remove_numbering_from_title("标题一、"), "标题一、");
  }

  #[test]
  fn test_parse_whitespace_handling() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"# 标题1

    
## 标题2
   
内容行
   
"#;

    let mst = parser.parse(content).unwrap();
    let headers = mst.get_headers();

    assert_eq!(headers.len(), 2);
    assert_eq!(headers[0].title.as_ref().unwrap(), "标题1");
    assert_eq!(headers[1].title.as_ref().unwrap(), "标题2");
  }

  #[test]
  fn test_image_parsing_edge_cases() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"![](https://example.com/empty-alt.png)
![alt](not-a-url)
![alt](https://example.com/image.png)
![alt](https://example.com/image2.jpg "title")
<img src="https://example.com/html-img.png" alt="html alt">
<img src="local-image.png" alt="local">"#;

    let mst = parser.parse(content).unwrap();

    let mut image_count = 0;
    mst.walk(&mut |node| {
      if node.is_image() {
        image_count += 1;
      }
    });

    assert_eq!(image_count, 4); // 只有远程图片被解析
  }

  #[test]
  fn test_parser_default() {
    let parser = MarkdownParser::default();
    let content = "# Test";
    let mst = parser.parse(content).unwrap();
    let headers = mst.get_headers();
    assert_eq!(headers.len(), 1);
  }

  #[test]
  fn test_line_numbers() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"# 标题1
内容1
## 标题2
内容2"#;

    let mst = parser.parse(content).unwrap();

    // 验证行号
    let mut line_numbers = Vec::new();
    mst.walk(&mut |node| {
      line_numbers.push(node.line_number);
    });

    // 根节点行号为0，其他节点行号从1开始
    assert_eq!(line_numbers[0], 0); // 根节点
    assert!(line_numbers.iter().skip(1).all(|&n| n > 0)); // 其他节点行号大于0
  }

  #[test]
  fn test_malformed_headers() {
    let parser = MarkdownParser::new().unwrap();
    let content = r#"#标题1
# 
##标题2
###   标题3   
####
#####标题5"#;

    let mst = parser.parse(content).unwrap();
    let headers = mst.get_headers();

    // 只有符合格式的标题会被解析
    assert_eq!(headers.len(), 2); // 只有 "###   标题3   " 和 "#####标题5" 符合格式
  }

  /// 测试行内图片解析和渲染的问题
  #[test]
  fn test_inline_image_parsing_and_rendering() {
    let parser = MarkdownParser::new().unwrap();
    let renderer = MarkdownRenderer::new();

    // 测试包含行内图片的内容
    let content = "前面有文字，![测试图片3](https://picsum.photos/200/300)，后面有文字";

    let mst = parser.parse(content).unwrap();

    // 检查解析结果
    println!("=== 解析结果 ===");
    println!("MST结构:");
    println!("{}", mst);

    // 统计节点类型
    let mut content_nodes = 0;
    let mut image_nodes = 0;

    mst.walk(&mut |node| match &node.node_type {
      NodeType::Content(content) => {
        content_nodes += 1;
        println!("内容节点 {}: {}", node.line_number, content);
      }
      NodeType::Image(image_info) => {
        image_nodes += 1;
        println!("图片节点 {}: {} (alt: {})", node.line_number, image_info.original_url, image_info.alt_text);
      }
      _ => {}
    });

    println!("内容节点数量: {}", content_nodes);
    println!("图片节点数量: {}", image_nodes);

    // 渲染结果
    let rendered = renderer.render(&mst);
    println!("=== 渲染结果 ===");
    println!("{}", rendered);

    // 验证问题：应该只有一行，但实际会有两行
    let lines: Vec<&str> = rendered.lines().collect();
    println!("渲染行数: {}", lines.len());
    for (i, line) in lines.iter().enumerate() {
      println!("行 {}: {}", i + 1, line);
    }

    // 验证主要问题已修复：只有一行输出，不再重复
    assert_eq!(lines.len(), 1, "行内图片应该只渲染为一行");
    assert_eq!(content_nodes, 1, "应该只有一个内容节点");
    assert_eq!(image_nodes, 0, "不应该有单独的图片节点");

    // 验证内容包含原始图片引用（图片本地化会在实际使用时处理）
    assert!(lines[0].contains("![测试图片3]"), "应该包含图片引用");
    assert!(lines[0].contains("前面有文字"), "应该包含前面的文字");
    assert!(lines[0].contains("后面有文字"), "应该包含后面的文字");
  }
}
