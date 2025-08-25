//! 编号生成器
//!
//! 基于 MST 结构为标题生成各种格式的编号

use crate::mst::{MSTNode, NumberingConfig, NumberingInfo};

/// 编号生成器
pub struct NumberingGenerator {
  config: NumberingConfig,
}

impl NumberingGenerator {
  /// 创建新的编号生成器
  pub fn new(config: NumberingConfig) -> Self {
    Self { config }
  }

  /// 为 MST 中的所有标题生成编号
  pub fn generate_numbering(&self, mst: &mut MSTNode) {
    let mut counters = vec![0; 6]; // 支持6级标题的计数器
    mst.apply_to_headers(&mut |node| {
      if let Some(level) = node.header_level() {
        self.apply_numbering_to_node(node, &mut counters, level);
      }
    });
  }

  /// 为单个节点应用编号
  fn apply_numbering_to_node(&self, node: &mut MSTNode, counters: &mut Vec<usize>, level: usize) {
    // 如果设置忽略一级标题且当前是一级标题，不生成编号
    if self.config.ignore_h1 && level == 1 {
      node.numbering = None;
      return;
    }

    // 更新当前级别的计数器
    let effective_level = if self.config.ignore_h1 && level > 1 {
      level - 1 // 忽略H1时，H2变成第1级
    } else {
      level
    };

    counters[effective_level - 1] += 1;

    // 重置子级的计数器
    for i in effective_level..6 {
      counters[i] = 0;
    }

    // 生成编号路径
    let mut path = Vec::new();
    let start_level = 0;

    for i in start_level..effective_level {
      if counters[i] > 0 {
        path.push(counters[i]);
      }
    }

    // 生成格式化的编号字符串
    let formatted = self.format_numbering(&path, effective_level);

    node.numbering = Some(NumberingInfo { path, formatted });
  }

  /// 格式化编号
  fn format_numbering(&self, path: &[usize], level: usize) -> String {
    if path.is_empty() {
      return String::new();
    }

    if self.config.use_chinese_number && self.config.use_arabic_number_for_sublevel {
      // 混合编号：第一级用中文，子级用阿拉伯数字
      if path.len() == 1 {
        // 只有一级编号，使用中文
        format!("{}、", to_chinese_number(path[0]))
      } else {
        // 多级编号：子级使用阿拉伯数字（不包含第一级的中文前缀）
        let rest_parts: Vec<String> = path[1..].iter().map(|n| n.to_string()).collect();
        format!("{}. ", rest_parts.join("."))
      }
    } else if self.config.use_chinese_number {
      // 全部使用中文编号
      let chinese_parts: Vec<String> = path.iter().map(|n| to_chinese_number(*n)).collect();
      format!("{}、", chinese_parts.join("、"))
    } else {
      // 全部使用阿拉伯数字编号
      let arabic_parts: Vec<String> = path.iter().map(|n| n.to_string()).collect();
      format!("{}. ", arabic_parts.join("."))
    }
  }
}

/// 将阿拉伯数字转换为中文数字
fn to_chinese_number(num: usize) -> String {
  let chinese_digits = ["", "一", "二", "三", "四", "五", "六", "七", "八", "九"];

  if num == 0 {
    return "零".to_string();
  }

  if num < 10 {
    return chinese_digits[num].to_string();
  }

  if num < 100 {
    let tens = num / 10;
    let ones = num % 10;
    if tens == 1 {
      if ones == 0 {
        return "十".to_string();
      } else {
        return format!("十{}", chinese_digits[ones]);
      }
    } else {
      if ones == 0 {
        return format!("{}十", chinese_digits[tens]);
      } else {
        return format!("{}十{}", chinese_digits[tens], chinese_digits[ones]);
      }
    }
  }

  // 对于更大的数字，简化处理
  num.to_string()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::mst::{MSTNode, NodeType};

  fn create_test_mst() -> MSTNode {
    let mut root = MSTNode::new_root();

    let mut h1 = MSTNode::new_header(1, "标题1".to_string(), "标题1".to_string(), "# 标题1".to_string(), 1);
    let mut h2_1 = MSTNode::new_header(2, "子标题1".to_string(), "子标题1".to_string(), "## 子标题1".to_string(), 2);
    let h3_1 = MSTNode::new_header(3, "子子标题1".to_string(), "子子标题1".to_string(), "### 子子标题1".to_string(), 3);
    let h3_2 = MSTNode::new_header(3, "子子标题2".to_string(), "子子标题2".to_string(), "### 子子标题2".to_string(), 4);
    let h2_2 = MSTNode::new_header(2, "子标题2".to_string(), "子标题2".to_string(), "## 子标题2".to_string(), 5);
    let h1_2 = MSTNode::new_header(1, "标题2".to_string(), "标题2".to_string(), "# 标题2".to_string(), 6);

    h2_1.add_child(h3_1);
    h2_1.add_child(h3_2);
    h1.add_child(h2_1);
    h1.add_child(h2_2);
    root.add_child(h1);
    root.add_child(h1_2);

    root
  }

  #[test]
  fn test_to_chinese_number() {
    assert_eq!(to_chinese_number(0), "零");
    assert_eq!(to_chinese_number(1), "一");
    assert_eq!(to_chinese_number(5), "五");
    assert_eq!(to_chinese_number(9), "九");
    assert_eq!(to_chinese_number(10), "十");
    assert_eq!(to_chinese_number(11), "十一");
    assert_eq!(to_chinese_number(15), "十五");
    assert_eq!(to_chinese_number(20), "二十");
    assert_eq!(to_chinese_number(21), "二十一");
    assert_eq!(to_chinese_number(99), "九十九");
    assert_eq!(to_chinese_number(100), "100"); // 超过99的数字返回阿拉伯数字
  }

  #[test]
  fn test_arabic_numbering() {
    let mut mst = create_test_mst();
    let config = NumberingConfig { ignore_h1: false, use_chinese_number: false, use_arabic_number_for_sublevel: false };

    let generator = NumberingGenerator::new(config);
    generator.generate_numbering(&mut mst);

    let headers = mst.get_headers();
    assert_eq!(headers[0].numbering.as_ref().unwrap().formatted, "1. ");
    assert_eq!(headers[1].numbering.as_ref().unwrap().formatted, "1.1. ");
    assert_eq!(headers[2].numbering.as_ref().unwrap().formatted, "1.1.1. ");
    assert_eq!(headers[3].numbering.as_ref().unwrap().formatted, "1.1.2. ");
    assert_eq!(headers[4].numbering.as_ref().unwrap().formatted, "1.2. ");
    assert_eq!(headers[5].numbering.as_ref().unwrap().formatted, "2. ");
  }

  #[test]
  fn test_chinese_numbering() {
    let mut mst = create_test_mst();
    let config = NumberingConfig { ignore_h1: false, use_chinese_number: true, use_arabic_number_for_sublevel: false };

    let generator = NumberingGenerator::new(config);
    generator.generate_numbering(&mut mst);

    let headers = mst.get_headers();
    assert_eq!(headers[0].numbering.as_ref().unwrap().formatted, "一、");
    assert_eq!(headers[1].numbering.as_ref().unwrap().formatted, "一、一、");
    assert_eq!(headers[2].numbering.as_ref().unwrap().formatted, "一、一、一、");
    assert_eq!(headers[3].numbering.as_ref().unwrap().formatted, "一、一、二、");
    assert_eq!(headers[4].numbering.as_ref().unwrap().formatted, "一、二、");
    assert_eq!(headers[5].numbering.as_ref().unwrap().formatted, "二、");
  }

  #[test]
  fn test_mixed_numbering() {
    let mut mst = create_test_mst();
    let config = NumberingConfig { ignore_h1: false, use_chinese_number: true, use_arabic_number_for_sublevel: true };

    let generator = NumberingGenerator::new(config);
    generator.generate_numbering(&mut mst);

    let headers = mst.get_headers();
    assert_eq!(headers[0].numbering.as_ref().unwrap().formatted, "一、");
    assert_eq!(headers[1].numbering.as_ref().unwrap().formatted, "1. ");
    assert_eq!(headers[2].numbering.as_ref().unwrap().formatted, "1.1. ");
    assert_eq!(headers[3].numbering.as_ref().unwrap().formatted, "1.2. ");
    assert_eq!(headers[4].numbering.as_ref().unwrap().formatted, "2. ");
    assert_eq!(headers[5].numbering.as_ref().unwrap().formatted, "二、");
  }

  #[test]
  fn test_ignore_h1() {
    let mut mst = create_test_mst();
    let config = NumberingConfig { ignore_h1: true, use_chinese_number: false, use_arabic_number_for_sublevel: false };

    let generator = NumberingGenerator::new(config);
    generator.generate_numbering(&mut mst);

    let headers = mst.get_headers();
    assert!(headers[0].numbering.is_none()); // H1 should have no numbering
    assert_eq!(headers[1].numbering.as_ref().unwrap().formatted, "1. ");
    assert_eq!(headers[2].numbering.as_ref().unwrap().formatted, "1.1. ");
    assert_eq!(headers[3].numbering.as_ref().unwrap().formatted, "1.2. ");
    assert_eq!(headers[4].numbering.as_ref().unwrap().formatted, "2. ");
    assert!(headers[5].numbering.is_none()); // H1 should have no numbering
  }

  #[test]
  fn test_ignore_h1_mixed() {
    let mut mst = create_test_mst();
    let config = NumberingConfig { ignore_h1: true, use_chinese_number: true, use_arabic_number_for_sublevel: true };

    let generator = NumberingGenerator::new(config);
    generator.generate_numbering(&mut mst);

    let headers = mst.get_headers();
    assert!(headers[0].numbering.is_none()); // H1 should have no numbering
    assert_eq!(headers[1].numbering.as_ref().unwrap().formatted, "一、");
    assert_eq!(headers[2].numbering.as_ref().unwrap().formatted, "1. ");
    assert_eq!(headers[3].numbering.as_ref().unwrap().formatted, "2. ");
    assert_eq!(headers[4].numbering.as_ref().unwrap().formatted, "二、");
    assert!(headers[5].numbering.is_none()); // H1 should have no numbering
  }
}
