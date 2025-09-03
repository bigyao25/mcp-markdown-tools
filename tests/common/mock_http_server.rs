//! 测试工具和辅助函数

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
