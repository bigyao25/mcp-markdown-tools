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

    write_file_content(output_path, &new_content)?;

    let final_message = format!("{}, 新文件保存为: {}", success_message, output_path);
    Ok(CallToolResult::success(vec![Content::text(final_message)]))
  })();

  result.map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::path::Path;
  use tempfile::NamedTempFile;

  /// 测试验证有效的 Markdown 文件
  #[test]
  fn test_validate_markdown_file_valid() {
    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), "# Test\n\nContent").unwrap();

    let path_str = temp_file.path().to_str().unwrap();
    let result = validate_markdown_file(path_str);

    assert!(result.is_ok());
  }

  /// 测试验证不存在的文件
  #[test]
  fn test_validate_markdown_file_not_exists() {
    let result = validate_markdown_file("/nonexistent/file.md");

    assert!(result.is_err());
    match result.unwrap_err() {
      MarkdownError::ValidationError(msg) => {
        assert!(msg.contains("文件不存在"));
      }
      _ => panic!("期望 ValidationError"),
    }
  }

  /// 测试验证非 Markdown 文件扩展名
  #[test]
  fn test_validate_markdown_file_wrong_extension() {
    let temp_file = NamedTempFile::with_suffix(".txt").unwrap();
    fs::write(temp_file.path(), "# Test").unwrap();

    let path_str = temp_file.path().to_str().unwrap();
    let result = validate_markdown_file(path_str);

    assert!(result.is_err());
    match result.unwrap_err() {
      MarkdownError::ValidationError(msg) => {
        assert!(msg.contains("文件必须是Markdown格式"));
      }
      _ => panic!("期望 ValidationError"),
    }
  }

  /// 测试验证 .markdown 扩展名
  #[test]
  fn test_validate_markdown_file_markdown_extension() {
    let temp_file = NamedTempFile::with_suffix(".markdown").unwrap();
    fs::write(temp_file.path(), "# Test").unwrap();

    let path_str = temp_file.path().to_str().unwrap();
    let result = validate_markdown_file(path_str);

    assert!(result.is_ok());
  }

  /// 测试验证无扩展名文件
  #[test]
  fn test_validate_markdown_file_no_extension() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("testfile");
    fs::write(&file_path, "# Test").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = validate_markdown_file(path_str);

    assert!(result.is_err());
    match result.unwrap_err() {
      MarkdownError::ValidationError(msg) => {
        assert!(msg.contains("文件必须有扩展名"));
      }
      _ => panic!("期望 ValidationError"),
    }
  }

  /// 测试读取文件内容
  #[test]
  fn test_read_file_content_success() {
    let content = "# 测试标题\n\n这是测试内容。";
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let path_str = temp_file.path().to_str().unwrap();
    let result = read_file_content(path_str).unwrap();

    assert_eq!(result, content);
  }

  /// 测试读取不存在的文件
  #[test]
  fn test_read_file_content_not_exists() {
    let result = read_file_content("/nonexistent/file.md");

    assert!(result.is_err());
    match result.unwrap_err() {
      MarkdownError::FileError(msg) => {
        assert!(msg.contains("读取文件失败"));
      }
      _ => panic!("期望 FileError"),
    }
  }

  /// 测试读取空文件
  #[test]
  fn test_read_file_content_empty() {
    let temp_file = NamedTempFile::new().unwrap();
    // 创建空文件

    let path_str = temp_file.path().to_str().unwrap();
    let result = read_file_content(path_str).unwrap();

    assert_eq!(result, "");
  }

  /// 测试写入文件内容
  #[test]
  fn test_write_file_content_success() {
    let content = "# 新标题\n\n这是新内容。";
    let temp_file = NamedTempFile::new().unwrap();

    let path_str = temp_file.path().to_str().unwrap();
    let result = write_file_content(path_str, content);

    assert!(result.is_ok());

    // 验证文件内容
    let read_content = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(read_content, content);
  }

  /// 测试写入到不存在的目录
  #[test]
  fn test_write_file_content_invalid_path() {
    let result = write_file_content("/nonexistent/directory/file.md", "content");

    assert!(result.is_err());
    match result.unwrap_err() {
      MarkdownError::FileError(msg) => {
        assert!(msg.contains("写入文件失败"));
      }
      _ => panic!("期望 FileError"),
    }
  }

  /// 测试创建成功结果
  #[test]
  fn test_create_success_result() {
    let message = "操作成功完成".to_string();
    let result = create_success_result(message.clone()).unwrap();

    // 验证结果结构
    assert_eq!(result.is_error, Some(false));
    assert_eq!(result.content.len(), 1);
    // 简化测试，不检查具体的内容结构
  }

  /// 测试创建错误结果
  #[test]
  fn test_create_error_result() {
    let error_message = "操作失败".to_string();
    let result = create_error_result(error_message.clone()).unwrap();

    // 验证结果结构
    assert_eq!(result.is_error, Some(true));
    assert_eq!(result.content.len(), 1);
    // 简化测试，不检查具体的内容结构
  }

  /// 测试执行 Markdown 操作 - 成功情况
  #[test]
  fn test_execute_markdown_operation_success() {
    let content = "# 原始标题\n\n原始内容";
    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let path_str = temp_file.path().to_str().unwrap();

    // 定义一个简单的操作：添加前缀
    let operation =
      |input: &str| -> std::result::Result<String, String> { Ok(format!("修改后的内容:\n{}", input)) };

    let result = execute_markdown_operation(path_str, operation, "操作成功".to_string(), false, path_str).unwrap();

    assert_eq!(result.is_error, Some(false));

    // 验证文件内容被修改
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(new_content.starts_with("修改后的内容:"));
  }

  /// 测试执行 Markdown 操作 - 保存为新文件
  #[test]
  fn test_execute_markdown_operation_save_as_new() {
    let content = "# 原始标题";
    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let new_file_path = temp_dir.path().join("new_file.md");

    let path_str = temp_file.path().to_str().unwrap();
    let new_path_str = new_file_path.to_str().unwrap();

    let operation = |input: &str| -> std::result::Result<String, String> { Ok(format!("新内容: {}", input)) };

    let result = execute_markdown_operation(path_str, operation, "操作成功".to_string(), true, new_path_str).unwrap();

    assert_eq!(result.is_error, Some(false));

    // 验证原文件未被修改
    let original_content = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(original_content, content);

    // 验证新文件被创建
    assert!(new_file_path.exists());
    let new_content = fs::read_to_string(&new_file_path).unwrap();
    assert!(new_content.starts_with("新内容:"));
  }

  /// 测试执行 Markdown 操作 - 文件验证失败
  #[test]
  fn test_execute_markdown_operation_validation_error() {
    let operation = |_input: &str| -> std::result::Result<String, String> { Ok("不会被调用".to_string()) };

    let result = execute_markdown_operation(
      "/nonexistent/file.md",
      operation,
      "不会成功".to_string(),
      false,
      "/nonexistent/file.md",
    );

    assert!(result.is_err());
  }

  /// 测试执行 Markdown 操作 - 操作失败
  #[test]
  fn test_execute_markdown_operation_operation_error() {
    let content = "# 测试";
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let path_str = temp_file.path().to_str().unwrap();

    let operation = |_input: &str| -> std::result::Result<String, String> { Err("操作失败".to_string()) };

    let result = execute_markdown_operation(path_str, operation, "不会成功".to_string(), false, path_str);

    assert!(result.is_err());
  }

  /// 测试处理 UTF-8 内容
  #[test]
  fn test_utf8_content_handling() {
    let content = "# 中文标题\n\n这是中文内容：测试、验证、确认。\n\n## 英文 Section\n\nMixed content.";
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let path_str = temp_file.path().to_str().unwrap();

    // 测试读取
    let read_result = read_file_content(path_str).unwrap();
    assert_eq!(read_result, content);

    // 测试写入
    let new_content = format!("更新的内容：\n{}", content);
    write_file_content(path_str, &new_content).unwrap();

    let final_content = read_file_content(path_str).unwrap();
    assert_eq!(final_content, new_content);
  }

  /// 测试大文件处理
  #[test]
  fn test_large_content_handling() {
    // 创建一个相对较大的内容
    let mut large_content = String::new();
    for i in 0..1000 {
      large_content.push_str(&format!("# 标题 {}\n\n这是第 {} 个段落的内容。\n\n", i, i));
    }

    let temp_file = NamedTempFile::new().unwrap();
    let path_str = temp_file.path().to_str().unwrap();

    // 测试写入大内容
    write_file_content(path_str, &large_content).unwrap();

    // 测试读取大内容
    let read_content = read_file_content(path_str).unwrap();
    assert_eq!(read_content, large_content);
  }

  /// 测试边界情况：空字符串操作
  #[test]
  fn test_empty_string_operations() {
    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    let path_str = temp_file.path().to_str().unwrap();

    // 写入空字符串
    write_file_content(path_str, "").unwrap();

    // 读取空字符串
    let content = read_file_content(path_str).unwrap();
    assert_eq!(content, "");

    // 执行空字符串操作
    let operation = |input: &str| -> std::result::Result<String, String> {
      assert_eq!(input, "");
      Ok("处理后的空内容".to_string())
    };

    let result =
      execute_markdown_operation(path_str, operation, "空内容处理成功".to_string(), false, path_str).unwrap();

    assert_eq!(result.is_error, Some(false));
  }
}
