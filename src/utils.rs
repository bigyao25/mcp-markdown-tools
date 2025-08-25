use rmcp::{ErrorData as McpError, model::*};
use std::fs;
use std::path::Path;

/// 验证文件路径并检查是否为有效的 Markdown 文件
pub fn validate_markdown_file(file_path: &str) -> Result<(), String> {
    let path = Path::new(file_path);

    // 检查文件是否存在
    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    // 检查是否为markdown文件
    if let Some(extension) = path.extension() {
        if extension != "md" && extension != "markdown" {
            return Err("文件必须是Markdown格式 (.md 或 .markdown)".to_string());
        }
    } else {
        return Err("文件必须有扩展名".to_string());
    }

    Ok(())
}

/// 读取文件内容
pub fn read_file_content(file_path: &str) -> Result<String, String> {
    match fs::read_to_string(file_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("读取文件失败: {}", e)),
    }
}

/// 写入文件内容
pub fn write_file_content(file_path: &str, content: &str) -> Result<(), String> {
    match fs::write(file_path, content) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("写入文件失败: {}", e)),
    }
}

/// 创建成功的工具调用结果
pub fn create_success_result(message: String) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(message)]))
}

/// 创建错误的工具调用结果
pub fn create_error_result(error_message: String) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![Content::text(error_message)]))
}

/// 执行 Markdown 文件操作的通用流程
pub fn execute_markdown_operation<F>(
    file_path: &str,
    operation: F,
    success_message: String,
) -> Result<CallToolResult, McpError>
where
    F: FnOnce(&str) -> Result<String, String>,
{
    // 验证文件
    if let Err(error) = validate_markdown_file(file_path) {
        return create_error_result(error);
    }

    // 读取文件内容
    let content = match read_file_content(file_path) {
        Ok(content) => content,
        Err(error) => return create_error_result(error),
    };

    // 执行操作
    let new_content = match operation(&content) {
        Ok(new_content) => new_content,
        Err(error) => return create_error_result(error),
    };

    // 写入文件
    if let Err(error) = write_file_content(file_path, &new_content) {
        return create_error_result(error);
    }

    create_success_result(success_message)
}
