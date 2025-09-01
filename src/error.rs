use rmcp::ErrorData as McpError;
use std::fmt;

#[derive(Debug)]
pub enum MarkdownError {
  ParseError(String),
  FileError(String),
  ValidationError(String),
  ConfigError(String),
}

impl fmt::Display for MarkdownError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      MarkdownError::ParseError(msg) => write!(f, "解析错误: {}", msg),
      MarkdownError::FileError(msg) => write!(f, "文件错误: {}", msg),
      MarkdownError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
      MarkdownError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
    }
  }
}

impl std::error::Error for MarkdownError {}

impl From<MarkdownError> for McpError {
  fn from(err: MarkdownError) -> Self {
    match err {
      MarkdownError::ParseError(msg) | MarkdownError::ValidationError(msg) | MarkdownError::ConfigError(msg) => {
        McpError::invalid_params(msg, None)
      }
      MarkdownError::FileError(msg) => McpError::internal_error(msg, None),
    }
  }
}

pub type Result<T> = std::result::Result<T, MarkdownError>;

#[cfg(test)]
mod tests {
  use super::*;
  use std::error::Error;

  /// 测试 MarkdownError 的 Display 实现
  #[test]
  fn test_markdown_error_display() {
    let parse_error = MarkdownError::ParseError("解析失败".to_string());
    assert_eq!(parse_error.to_string(), "解析错误: 解析失败");

    let file_error = MarkdownError::FileError("文件不存在".to_string());
    assert_eq!(file_error.to_string(), "文件错误: 文件不存在");

    let validation_error = MarkdownError::ValidationError("验证失败".to_string());
    assert_eq!(validation_error.to_string(), "验证错误: 验证失败");

    let config_error = MarkdownError::ConfigError("配置无效".to_string());
    assert_eq!(config_error.to_string(), "配置错误: 配置无效");
  }

  /// 测试 MarkdownError 的 Debug 实现
  #[test]
  fn test_markdown_error_debug() {
    let error = MarkdownError::ParseError("测试错误".to_string());
    let debug_output = format!("{:?}", error);
    assert!(debug_output.contains("ParseError"));
    assert!(debug_output.contains("测试错误"));
  }

  /// 测试 MarkdownError 实现了 std::error::Error trait
  #[test]
  fn test_markdown_error_is_error() {
    let error = MarkdownError::FileError("测试".to_string());

    // 测试 Error trait 方法
    assert!(error.source().is_none());

    // 测试可以转换为 Box<dyn Error>
    let _boxed_error: Box<dyn Error> = Box::new(error);
  }

  /// 测试 ParseError 转换为 McpError
  #[test]
  fn test_parse_error_to_mcp_error() {
    let markdown_error = MarkdownError::ParseError("解析失败".to_string());
    let mcp_error: McpError = markdown_error.into();

    // 验证错误消息
    assert_eq!(mcp_error.message, "解析失败");
    assert!(mcp_error.data.is_none());
  }

  /// 测试 ValidationError 转换为 McpError
  #[test]
  fn test_validation_error_to_mcp_error() {
    let markdown_error = MarkdownError::ValidationError("验证失败".to_string());
    let mcp_error: McpError = markdown_error.into();

    assert_eq!(mcp_error.message, "验证失败");
    assert!(mcp_error.data.is_none());
  }

  /// 测试 ConfigError 转换为 McpError
  #[test]
  fn test_config_error_to_mcp_error() {
    let markdown_error = MarkdownError::ConfigError("配置错误".to_string());
    let mcp_error: McpError = markdown_error.into();

    assert_eq!(mcp_error.message, "配置错误");
    assert!(mcp_error.data.is_none());
  }

  /// 测试 FileError 转换为 McpError
  #[test]
  fn test_file_error_to_mcp_error() {
    let markdown_error = MarkdownError::FileError("文件操作失败".to_string());
    let mcp_error: McpError = markdown_error.into();

    assert_eq!(mcp_error.message, "文件操作失败");
    assert!(mcp_error.data.is_none());
  }

  /// 测试 Result 类型别名
  #[test]
  fn test_result_type_alias() {
    // 测试成功情况
    let success_result: Result<String> = Ok("成功".to_string());
    assert!(success_result.is_ok());
    assert_eq!(success_result.unwrap(), "成功");

    // 测试错误情况
    let error_result: Result<String> = Err(MarkdownError::ParseError("失败".to_string()));
    assert!(error_result.is_err());

    match error_result.unwrap_err() {
      MarkdownError::ParseError(msg) => assert_eq!(msg, "失败"),
      _ => panic!("期望 ParseError"),
    }
  }

  /// 测试错误链
  #[test]
  fn test_error_chaining() {
    let error = MarkdownError::FileError("底层错误".to_string());

    // 模拟错误传播
    let propagated_error = match error {
      MarkdownError::FileError(msg) => MarkdownError::ValidationError(format!("由于文件错误导致的验证失败: {}", msg)),
      other => other,
    };

    assert_eq!(propagated_error.to_string(), "验证错误: 由于文件错误导致的验证失败: 底层错误");
  }

  /// 测试空消息的错误
  #[test]
  fn test_empty_message_error() {
    let error = MarkdownError::ParseError("".to_string());
    assert_eq!(error.to_string(), "解析错误: ");
  }

  /// 测试包含特殊字符的错误消息
  #[test]
  fn test_special_characters_in_error_message() {
    let message = "错误：文件 'test.md' 在第 10 行包含无效字符 \"\\n\"";
    let error = MarkdownError::ParseError(message.to_string());
    assert_eq!(error.to_string(), format!("解析错误: {}", message));
  }

  /// 测试多字节 Unicode 字符的错误消息
  #[test]
  fn test_unicode_error_message() {
    let message = "解析错误：文档包含emoji 🚀 和中文字符";
    let error = MarkdownError::ValidationError(message.to_string());
    assert_eq!(error.to_string(), format!("验证错误: {}", message));
  }

  /// 测试长错误消息
  #[test]
  fn test_long_error_message() {
    let long_message = "这是一个非常长的错误消息，".repeat(100);
    let error = MarkdownError::ConfigError(long_message.clone());
    assert_eq!(error.to_string(), format!("配置错误: {}", long_message));
  }

  /// 测试错误类型的相等性
  #[test]
  fn test_error_type_matching() {
    let parse_error = MarkdownError::ParseError("test".to_string());
    let file_error = MarkdownError::FileError("test".to_string());
    let validation_error = MarkdownError::ValidationError("test".to_string());
    let config_error = MarkdownError::ConfigError("test".to_string());

    // 测试模式匹配
    match parse_error {
      MarkdownError::ParseError(_) => (),
      _ => panic!("应该匹配 ParseError"),
    }

    match file_error {
      MarkdownError::FileError(_) => (),
      _ => panic!("应该匹配 FileError"),
    }

    match validation_error {
      MarkdownError::ValidationError(_) => (),
      _ => panic!("应该匹配 ValidationError"),
    }

    match config_error {
      MarkdownError::ConfigError(_) => (),
      _ => panic!("应该匹配 ConfigError"),
    }
  }

  /// 测试错误转换的幂等性
  #[test]
  fn test_error_conversion_idempotency() {
    let original_error = MarkdownError::ParseError("原始错误".to_string());
    let mcp_error: McpError = original_error.into();

    // 验证转换后的属性
    assert_eq!(mcp_error.message, "原始错误");

    // 多次转换应该产生相同的结果
    let error2 = MarkdownError::ParseError("原始错误".to_string());
    let mcp_error2: McpError = error2.into();

    assert_eq!(mcp_error.message, mcp_error2.message);
  }

  /// 测试在函数中使用 Result 类型
  #[test]
  fn test_result_in_function() {
    fn parse_number(s: &str) -> Result<i32> {
      s.parse::<i32>().map_err(|_| MarkdownError::ParseError(format!("无法解析数字: {}", s)))
    }

    // 测试成功情况
    let result = parse_number("42");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    // 测试失败情况
    let result = parse_number("not_a_number");
    assert!(result.is_err());

    match result.unwrap_err() {
      MarkdownError::ParseError(msg) => {
        assert!(msg.contains("无法解析数字"));
        assert!(msg.contains("not_a_number"));
      }
      _ => panic!("期望 ParseError"),
    }
  }

  /// 测试错误的 Send 和 Sync traits
  #[test]
  fn test_error_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<MarkdownError>();
    assert_sync::<MarkdownError>();
  }
}
