//! 测试工具和辅助函数

use std::fs;
use std::path::Path;
use tempfile::{NamedTempFile, TempDir};

/// 测试固定数据和工具
pub struct TestFixtures;

impl TestFixtures {
  /// 获取示例 Markdown 内容
  pub fn sample_markdown() -> &'static str {
    r#"# 第一章 介绍

这是第一章的内容。

## 1.1 背景

这是背景介绍。

### 1.1.1 历史

这是历史部分。

## 1.2 目标

这是目标说明。

# 第二章 实现

这是第二章的内容。

## 2.1 架构

这是架构说明。

![示例图片](https://example.com/image.jpg "图片标题")

### 2.1.1 组件

这是组件说明。

<img src="https://example.com/another.png" alt="另一张图片" width="100">

## 2.2 实现细节

这是实现细节。
"#
  }

  /// 获取复杂的 Markdown 内容（用于测试边界情况）
  pub fn complex_markdown() -> &'static str {
    r#"# 一、主标题

前言内容，不是标题。

## 一、子标题

### 1.1 三级标题

#### 1.1.1 四级标题

##### 1.1.1.1 五级标题

###### 1.1.1.1.1 六级标题

## 二、另一个子标题

### 2.1 另一个三级标题

# 二、第二个主标题

## 1. 阿拉伯数字标题

### 1.1. 带点的标题

![图片1](https://example.com/img1.jpg)

![图片2](https://example.com/img2.png "带标题的图片")

<img src="https://example.com/img3.gif" alt="GIF图片" class="responsive">

普通内容行。

## 2. 第二个阿拉伯数字标题
"#
  }

  /// 获取包含图片的 Markdown 内容
  pub fn markdown_with_images() -> &'static str {
    r#"# 图片测试文档

## 第一节

这里有一张 Markdown 格式的图片：

![测试图片](https://httpbin.org/image/png)

## 第二节

这里有一张 HTML 格式的图片：

<img src="https://httpbin.org/image/jpeg" alt="JPEG图片" width="200">

## 第三节

这里有带标题的图片：

![带标题图片](https://httpbin.org/image/svg "SVG图片")

普通文本内容。
"#
  }

  /// 获取无效格式的 Markdown 内容
  pub fn invalid_markdown() -> &'static str {
    r#"# 正确的标题

##错误的标题（缺少空格）

### 正确的三级标题

####  错误的标题（多个空格）

# 

## 正确的二级标题

#### 跳级的标题（从二级直接跳到四级）
"#
  }

  /// 创建临时 Markdown 文件
  pub fn create_temp_markdown_file(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().expect("创建临时文件失败");
    fs::write(temp_file.path(), content).expect("写入临时文件失败");
    temp_file
  }

  /// 创建临时目录
  pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("创建临时目录失败")
  }

  /// 创建带有指定内容的临时 Markdown 文件，返回路径字符串
  pub fn create_temp_markdown_with_path(content: &str) -> (NamedTempFile, String) {
    let temp_file = Self::create_temp_markdown_file(content);
    let path = temp_file.path().to_string_lossy().to_string();
    (temp_file, path)
  }

  /// 验证文件内容是否匹配
  pub fn assert_file_content_matches(file_path: &str, expected_content: &str) {
    let actual_content = fs::read_to_string(file_path).expect("读取文件失败");
    assert_eq!(actual_content.trim(), expected_content.trim());
  }

  /// 验证文件是否存在
  pub fn assert_file_exists(file_path: &str) {
    assert!(Path::new(file_path).exists(), "文件不存在: {}", file_path);
  }

  /// 验证文件是否不存在
  pub fn assert_file_not_exists(file_path: &str) {
    assert!(!Path::new(file_path).exists(), "文件不应该存在: {}", file_path);
  }
}

/// HTTP 模拟服务器工具
#[cfg(feature = "mock-server")]
pub struct MockHttpServer {
  server: wiremock::MockServer,
}

#[cfg(feature = "mock-server")]
impl MockHttpServer {
  /// 创建新的模拟服务器
  pub async fn new() -> Self {
    let server = wiremock::MockServer::start().await;
    Self { server }
  }

  /// 获取服务器 URL
  pub fn url(&self) -> String {
    self.server.uri()
  }

  /// 模拟图片下载响应
  pub async fn mock_image_response(&self, path: &str, image_data: &[u8], content_type: &str) {
    use wiremock::matchers::path;
    use wiremock::{Mock, ResponseTemplate};

    Mock::given(path(path))
      .respond_with(ResponseTemplate::new(200).set_body_bytes(image_data).insert_header("content-type", content_type))
      .mount(&self.server)
      .await;
  }

  /// 模拟 404 响应
  pub async fn mock_404_response(&self, path: &str) {
    use wiremock::matchers::path;
    use wiremock::{Mock, ResponseTemplate};

    Mock::given(path(path)).respond_with(ResponseTemplate::new(404)).mount(&self.server).await;
  }
}

/// 测试断言宏
#[macro_export]
macro_rules! assert_error_contains {
    ($result:expr, $expected:expr) => {
        match $result {
            Err(e) => assert!(
                e.to_string().contains($expected),
                "错误信息 '{}' 不包含预期的 '{}'",
                e.to_string(),
                $expected
            ),
            Ok(_) => panic!("期望错误，但得到了成功结果"),
        }
    };
}

/// 创建测试用的配置
pub mod test_configs {
  use serde_json::{Map, Value};

  /// 创建生成章节编号的测试配置
  pub fn generate_chapter_config(file_path: &str) -> Map<String, Value> {
    let mut config = Map::new();
    config.insert("full_file_path".to_string(), Value::String(file_path.to_string()));
    config.insert("ignore_h1".to_string(), Value::Bool(false));
    config.insert("use_chinese_number".to_string(), Value::Bool(false));
    config.insert("use_arabic_number_for_sublevel".to_string(), Value::Bool(true));
    config.insert("save_as_new_file".to_string(), Value::Bool(false));
    config
  }

  /// 创建移除章节编号的测试配置
  pub fn remove_chapter_config(file_path: &str) -> Map<String, Value> {
    let mut config = Map::new();
    config.insert("full_file_path".to_string(), Value::String(file_path.to_string()));
    config.insert("save_as_new_file".to_string(), Value::Bool(false));
    config
  }

  /// 创建检查标题的测试配置
  pub fn check_heading_config(file_path: &str) -> Map<String, Value> {
    let mut config = Map::new();
    config.insert("full_file_path".to_string(), Value::String(file_path.to_string()));
    config
  }

  /// 创建本地化图片的测试配置
  pub fn localize_images_config(file_path: &str, save_dir: &str) -> Map<String, Value> {
    let mut config = Map::new();
    config.insert("full_file_path".to_string(), Value::String(file_path.to_string()));
    config.insert("image_file_name_pattern".to_string(), Value::String("{index}-{hash}".to_string()));
    config.insert("save_to_dir".to_string(), Value::String(save_dir.to_string()));
    config
  }
}
