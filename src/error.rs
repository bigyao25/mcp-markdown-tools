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
