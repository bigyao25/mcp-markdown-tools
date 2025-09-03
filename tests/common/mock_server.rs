//! HTTP 模拟服务器工具
//!
//! 提供用于测试的 HTTP 模拟服务器功能

/// HTTP 模拟服务器工具
#[cfg(feature = "mock")]
pub struct MockHttpServer {
  server: wiremock::MockServer,
}

#[cfg(feature = "mock")]
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
  pub async fn mock_image_response(&self, path_str: &str, image_data: &[u8], content_type: &str) {
    use wiremock::matchers::path;
    use wiremock::{Mock, ResponseTemplate};

    Mock::given(path(path_str))
      .respond_with(ResponseTemplate::new(200).set_body_bytes(image_data).insert_header("content-type", content_type))
      .mount(&self.server)
      .await;
  }

  /// 模拟 404 响应
  pub async fn mock_404_response(&self, path_str: &str) {
    use wiremock::matchers::path;
    use wiremock::{Mock, ResponseTemplate};

    Mock::given(path(path_str)).respond_with(ResponseTemplate::new(404)).mount(&self.server).await;
  }

  pub async fn mock_basic_images(&self) {
    let jpg_data = self.create_test_jpg_data();
    let png_data = self.create_test_png_data();
    let webp_data = self.create_test_webp_data();
    let svg_data = self.create_test_svg_data();

    self.mock_image_response("/jpg", &jpg_data, "image/jpeg").await;
    self.mock_image_response("/png", &jpg_data, "image/png").await;
    self.mock_image_response("/webp", &jpg_data, "image/webp").await;
    self.mock_image_response("/svg", &jpg_data, "image/svg+xml").await;
  }

  /// 模拟多个图片响应 - 批量设置
  pub async fn mock_multiple_images(&self, count: usize) {
    let jpg_data = self.create_test_jpg_data();
    let png_data = self.create_test_png_data();
    let webp_data = self.create_test_webp_data();
    let svg_data = self.create_test_svg_data();

    for i in 1..=count {
      self.mock_image_response(&format!("/image{}.jpg", i), &jpg_data, "image/jpeg").await;
      self.mock_image_response(&format!("/image{}.png", i), &png_data, "image/png").await;
      self.mock_image_response(&format!("/image{}.webp", i), &webp_data, "image/webp").await;
      self.mock_image_response(&format!("/image{}.svg", i), &svg_data, "image/svg+xml").await;
    }
  }

  /// 创建测试用的 JPEG 数据
  fn create_test_jpg_data(&self) -> Vec<u8> {
    vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0xFF, 0xD9]
  }

  /// 创建测试用的 PNG 数据
  fn create_test_png_data(&self) -> Vec<u8> {
    vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
  }

  /// 创建测试用的 WebP 数据
  fn create_test_webp_data(&self) -> Vec<u8> {
    vec![
      0x52, 0x49, 0x46, 0x46, // "RIFF"
      0x00, 0x00, 0x00, 0x00, // 文件大小占位符
      0x57, 0x45, 0x42, 0x50, // "WEBP"
    ]
  }

  /// 创建测试用的 SVG 数据
  fn create_test_svg_data(&self) -> Vec<u8> {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="red"/></svg>"#
            .as_bytes()
            .to_vec()
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

/// 测试结果断言宏
#[macro_export]
macro_rules! assert_call_result_success {
  ($result:expr) => {
    match $result {
      Ok(call_result) => {
        assert_eq!(call_result.is_error, Some(false), "调用应该成功");
        assert!(!call_result.content.is_empty(), "结果内容不应为空");
      }
      Err(e) => panic!("调用失败: {:?}", e),
    }
  };
}

/// 测试结果错误断言宏
#[macro_export]
macro_rules! assert_call_result_error {
  ($result:expr) => {
    match $result {
      Ok(call_result) => {
        assert_eq!(call_result.is_error, Some(true), "调用应该失败");
      }
      Err(_) => {
        // 直接错误也是可以接受的
      }
    }
  };
}
