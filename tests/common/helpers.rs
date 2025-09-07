//! 测试辅助函数
//!
//! 提供通用的测试工具和辅助函数

use mcp_markdown_tools::config::*;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

/// 测试文件管理器
pub struct TestFileManager {
  pub temp_dir: TempDir,
}

impl TestFileManager {
  /// 创建新的测试文件管理器
  pub fn new() -> Self {
    Self { temp_dir: TempDir::new().expect("Failed to create temp directory") }
  }

  /// 创建测试 Markdown 文件
  pub fn create_md_file(&self, filename: &str, content: &str) -> std::path::PathBuf {
    let file_path = self.temp_dir.path().join(filename);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path
  }

  /// 创建临时 Markdown 文件
  pub fn create_temp_md_file(content: &str) -> NamedTempFile {
    let temp_file = NamedTempFile::with_suffix(".md").expect("Failed to create temp file");
    fs::write(temp_file.path(), content).expect("Failed to write temp file");
    temp_file
  }

  /// 获取资源目录路径
  pub fn assets_dir(&self) -> std::path::PathBuf {
    self.temp_dir.path().join("assets")
  }
}

/// 配置构建器 - 编号生成配置
pub struct NumberingConfigBuilder {
  config: GenerateChapterConfig,
}

impl NumberingConfigBuilder {
  pub fn new(file_path: &str) -> Self {
    Self {
      config: GenerateChapterConfig {
        full_file_path: file_path.to_string(),
        ignore_h1: false,
        use_chinese_number: false,
        use_arabic_number_for_sublevel: true,
        save_as_new_file: false,
        new_full_file_path: None,
      },
    }
  }

  pub fn ignore_h1(mut self, ignore: bool) -> Self {
    self.config.ignore_h1 = ignore;
    self
  }

  pub fn use_chinese_number(mut self, use_chinese: bool) -> Self {
    self.config.use_chinese_number = use_chinese;
    self
  }

  pub fn use_arabic_for_sublevel(mut self, use_arabic: bool) -> Self {
    self.config.use_arabic_number_for_sublevel = use_arabic;
    self
  }

  pub fn save_as_new_file(mut self, new_file_path: Option<String>) -> Self {
    self.config.save_as_new_file = new_file_path.is_some();
    self.config.new_full_file_path = new_file_path;
    self
  }

  pub fn build(self) -> GenerateChapterConfig {
    self.config
  }
}

/// 配置构建器 - 图片本地化配置
pub struct ImageLocalizationConfigBuilder {
  config: LocalizeImagesConfig,
}

impl ImageLocalizationConfigBuilder {
  pub fn new(file_path: &str) -> Self {
    Self {
      config: LocalizeImagesConfig {
        full_file_path: file_path.to_string(),
        image_file_name_pattern: "{index}-{hash}".to_string(),
        image_dir: "./assets/".to_string(),
        new_full_file_path: None,
      },
    }
  }

  pub fn file_name_pattern(mut self, pattern: &str) -> Self {
    self.config.image_file_name_pattern = pattern.to_string();
    self
  }

  pub fn image_dir(mut self, dir: &str) -> Self {
    self.config.image_dir = dir.to_string();
    self
  }

  pub fn new_file_path(mut self, path: Option<String>) -> Self {
    self.config.new_full_file_path = path;
    self
  }

  pub fn build(self) -> LocalizeImagesConfig {
    self.config
  }
}

/// 测试断言辅助函数
pub mod assertions {
  use std::fs;

  /// 断言文件包含指定内容
  pub fn assert_file_contains(file_path: &std::path::Path, expected: &str) {
    let content = fs::read_to_string(file_path).expect("Failed to read file");
    assert!(content.contains(expected), "文件内容不包含预期文本: '{}'\n实际内容: {}", expected, content);
  }

  /// 断言文件不包含指定内容
  pub fn assert_file_not_contains(file_path: &std::path::Path, unexpected: &str) {
    let content = fs::read_to_string(file_path).expect("Failed to read file");
    assert!(!content.contains(unexpected), "文件内容包含不应该存在的文本: '{}'\n实际内容: {}", unexpected, content);
  }

  /// 断言目录存在
  pub fn assert_dir_exists(dir_path: &std::path::Path) {
    assert!(dir_path.exists(), "目录不存在: {:?}", dir_path);
    assert!(dir_path.is_dir(), "路径不是目录: {:?}", dir_path);
  }

  /// 断言文件数量
  pub fn assert_file_count(dir_path: &std::path::Path, expected_count: usize) {
    let count = fs::read_dir(dir_path).expect("Failed to read directory").count();
    assert_eq!(count, expected_count, "目录中文件数量不匹配");
  }
}

/// 常用测试数据
pub mod test_data {
  /// 简单的测试文档
  pub const SIMPLE_DOC: &str = r#"# 第一章 介绍

## 背景

### 历史

一些历史内容。

## 目标

项目目标描述。

# 第二章 实现

## 技术选择

技术选择说明。
"#;

  /// 复杂的测试文档
  pub const COMPLEX_DOC: &str = r#"# 主标题

前言内容，不是标题。

## 子标题

### 三级标题

#### 四级标题

##### 五级标题

###### 六级标题

## 第二个子标题

内容...

# 第二个主标题

## 另一个子标题

更多内容...
"#;

  /// 包含图片的测试文档
  pub fn doc_with_images(base_url: &str) -> String {
    format!(
      r#"# 图片测试文档

## 第一节

这里有一张图片：

![测试图片]({}/jpg)

## 第二节

HTML 图片：

<img src="{}/png" alt="PNG图片" width="200">

## 第三节

带标题的图片：

![带标题图片]({}/webp "这是标题")
"#,
      base_url, base_url, base_url
    )
  }

  /// 无图片的测试文档
  pub const DOC_WITHOUT_IMAGES: &str = r#"# 无图片文档

## 第一节

这是普通文本内容。

## 第二节

这里也没有图片，只有文字。

### 子节

更多文字内容。
"#;
}
