use std::path::Path;

use crate::mst::NumberingConfig;
use crate::numbering::NumberingGenerator;
use crate::parser::MarkdownParser;
use crate::renderer::MarkdownRenderer;
use crate::utils::execute_markdown_operation;
use rmcp::{model::*, ErrorData as McpError};

pub struct MarkdownToolsImpl;

impl MarkdownToolsImpl {
  pub async fn generate_chapter_number_impl(
    file_path: &str,
    ignore_h1: bool,
    use_chinese_number: bool,
    use_arabic_number_for_sublevel: bool,
    save_as_new_file: bool,
    new_file_name: Option<&str>,
    default_suffix: &str,
  ) -> Result<CallToolResult, McpError> {
    let new_file_path = Self::generate_new_filename(file_path, new_file_name, default_suffix);

    execute_markdown_operation(
      file_path,
      |content| {
        // 使用新的 MST 架构
        let parser = MarkdownParser::new().map_err(|e| format!("创建解析器失败: {}", e))?;

        let mut mst = parser.parse(content).map_err(|e| format!("解析 Markdown 失败: {}", e))?;

        let config = NumberingConfig { ignore_h1, use_chinese_number, use_arabic_number_for_sublevel };

        let generator = NumberingGenerator::new(config);
        generator.generate_numbering(&mut mst);

        let renderer = MarkdownRenderer::new();
        let result = renderer.render_with_numbering(&mst);

        Ok(result)
      },
      format!("成功为文件 {} 生成章节编号", file_path),
      save_as_new_file,
      new_file_path.as_str(),
    )
  }

  pub async fn remove_all_chapter_numbers_impl(
    file_path: &str,
    save_as_new_file: bool,
    new_file_name: Option<&str>,
    default_suffix: &str,
  ) -> Result<CallToolResult, McpError> {
    let new_file_path = Self::generate_new_filename(file_path, new_file_name, default_suffix);

    execute_markdown_operation(
      file_path,
      |content| {
        // 使用新的 MST 架构
        let parser = MarkdownParser::new().map_err(|e| format!("创建解析器失败: {}", e))?;

        let mst = parser.parse(content).map_err(|e| format!("解析 Markdown 失败: {}", e))?;

        let renderer = MarkdownRenderer::new();
        let result = renderer.render_without_numbering(&mst);

        Ok(result)
      },
      format!("成功清除文件 {} 的所有章节编号", file_path),
      save_as_new_file, // remove_all_chapter_numbers 不需要另存为新文件
      new_file_path.as_str(),
    )
  }

  /// 生成新文件名
  fn generate_new_filename(file_path: &str, new_file_name: Option<&str>, default_suffix: &str) -> String {
    let path = Path::new(file_path);
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem().unwrap().to_str().unwrap();
    let extension = path.extension().unwrap().to_str().unwrap();

    let new_path = parent.join(match new_file_name {
      Some(name) => format!("{}.{}", name, extension),
      None => format!("{}_{}.{}", stem, default_suffix, extension),
    });
    new_path.to_str().unwrap().to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_generate_chapter_number_arabic() {
    // 这里可以添加集成测试
    // 由于需要文件系统操作，暂时跳过
  }

  #[tokio::test]
  async fn test_remove_all_chapter_numbers() {
    // 这里可以添加集成测试
    // 由于需要文件系统操作，暂时跳过
  }
}
