//! utils.rs 模块的单元测试

use mcp_markdown_tools::error::MarkdownError;
use mcp_markdown_tools::utils::*;
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

#[cfg(test)]
mod tests {
  use super::*;

  /// 测试验证有效的 Markdown 文件
  #[test]
  fn test_validate_markdown_file_valid() {
    let temp_file = NamedTempFile::new().unwrap();
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
    assert!(result.is_success);
    assert_eq!(result.content.len(), 1);

    if let rmcp::model::Content::Text { text } = &result.content[0] {
      assert_eq!(text, &message);
    } else {
      panic!("期望文本内容");
    }
  }

  /// 测试创建错误结果
  #[test]
  fn test_create_error_result() {
    let error_message = "操作失败".to_string();
    let result = create_error_result(error_message.clone()).unwrap();

    // 验证结果结构
    assert!(!result.is_success);
    assert_eq!(result.content.len(), 1);

    if let rmcp::model::Content::Text { text } = &result.content[0] {
      assert_eq!(text, &error_message);
    } else {
      panic!("期望文本内容");
    }
  }

  /// 测试执行 Markdown 操作 - 成功情况
  #[test]
  fn test_execute_markdown_operation_success() {
    let content = "# 原始标题\n\n原始内容";
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let path_str = temp_file.path().to_str().unwrap();

    // 定义一个简单的操作：添加前缀
    let operation = |input: &str| -> Result<String, String> { Ok(format!("修改后的内容:\n{}", input)) };

    let result = execute_markdown_operation(path_str, operation, "操作成功".to_string(), false, path_str).unwrap();

    assert!(result.is_success);

    // 验证文件内容被修改
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(new_content.starts_with("修改后的内容:"));
  }

  /// 测试执行 Markdown 操作 - 保存为新文件
  #[test]
  fn test_execute_markdown_operation_save_as_new() {
    let content = "# 原始标题";
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let new_file_path = temp_dir.path().join("new_file.md");

    let path_str = temp_file.path().to_str().unwrap();
    let new_path_str = new_file_path.to_str().unwrap();

    let operation = |input: &str| -> Result<String, String> { Ok(format!("新内容: {}", input)) };

    let result = execute_markdown_operation(path_str, operation, "操作成功".to_string(), true, new_path_str).unwrap();

    assert!(result.is_success);

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
    let operation = |_input: &str| -> Result<String, String> { Ok("不会被调用".to_string()) };

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

    let operation = |_input: &str| -> Result<String, String> { Err("操作失败".to_string()) };

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
    let temp_file = NamedTempFile::new().unwrap();
    let path_str = temp_file.path().to_str().unwrap();

    // 写入空字符串
    write_file_content(path_str, "").unwrap();

    // 读取空字符串
    let content = read_file_content(path_str).unwrap();
    assert_eq!(content, "");

    // 执行空字符串操作
    let operation = |input: &str| -> Result<String, String> {
      assert_eq!(input, "");
      Ok("处理后的空内容".to_string())
    };

    let result =
      execute_markdown_operation(path_str, operation, "空内容处理成功".to_string(), false, path_str).unwrap();

    assert!(result.is_success);
  }
}
