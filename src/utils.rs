use crate::error::{MarkdownError, Result};
use rmcp::{model::*, ErrorData as McpError};
use std::fs;
use std::path::Path;

/// 验证文件路径并检查是否为有效的 Markdown 文件
pub fn validate_markdown_file(full_file_path: &str) -> Result<()> {
  let path = Path::new(full_file_path);

  if !path.exists() {
    return Err(MarkdownError::ValidationError(format!("文件不存在: {}", full_file_path)));
  }

  if let Some(extension) = path.extension() {
    if extension != "md" && extension != "markdown" {
      return Err(MarkdownError::ValidationError("文件必须是Markdown格式 (.md 或 .markdown)".to_string()));
    }
  } else {
    return Err(MarkdownError::ValidationError("文件必须有扩展名".to_string()));
  }

  Ok(())
}

/// 读取文件内容
pub fn read_file_content(full_file_path: &str) -> Result<String> {
  fs::read_to_string(full_file_path).map_err(|e| MarkdownError::FileError(format!("读取文件失败: {}", e)))
}

/// 写入文件内容
pub fn write_file_content(full_file_path: &str, content: &str) -> Result<()> {
  fs::write(full_file_path, content).map_err(|e| MarkdownError::FileError(format!("写入文件失败: {}", e)))
}

/// 创建成功的工具调用结果
pub fn create_success_result(message: String) -> std::result::Result<CallToolResult, McpError> {
  Ok(CallToolResult::success(vec![Content::text(message)]))
}

/// 创建错误的工具调用结果
pub fn create_error_result(error_message: String) -> std::result::Result<CallToolResult, McpError> {
  Ok(CallToolResult::error(vec![Content::text(error_message)]))
}

/// 执行 Markdown 文件操作的通用流程
pub fn execute_markdown_operation<F>(
  full_file_path: &str,
  operation: F,
  success_message: String,
  save_as_new_file: bool,
  new_full_file_path: &str,
) -> std::result::Result<CallToolResult, McpError>
where
  F: FnOnce(&str) -> std::result::Result<String, String>,
{
  let result = (|| -> Result<CallToolResult> {
    validate_markdown_file(full_file_path)?;

    let content = read_file_content(full_file_path)?;

    let new_content = operation(&content).map_err(|e| MarkdownError::ParseError(e))?;

    let output_path = if save_as_new_file { new_full_file_path } else { full_file_path };
    std::eprintln!(
      "120010: save_as_new_file: {}, new_full_file_path: {}, output_path: {}",
      save_as_new_file,
      new_full_file_path,
      output_path
    );

    write_file_content(output_path, &new_content)?;

    let final_message = format!("{}, 新文件保存为: {}", success_message, output_path);
    Ok(CallToolResult::success(vec![Content::text(final_message)]))
  })();

  result.map_err(|e| e.into())
}
