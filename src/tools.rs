use std::path::Path;

use crate::config::{CheckHeadingConfig, GenerateChapterConfig, RemoveChapterConfig};
use crate::mst::NumberingConfig;
use crate::numbering::NumberingGenerator;
use crate::parser::MarkdownParser;
use crate::renderer::MarkdownRenderer;
use crate::utils::execute_markdown_operation;
use rmcp::{model::*, ErrorData as McpError};

pub struct MarkdownToolsImpl;

impl MarkdownToolsImpl {
  pub async fn generate_chapter_number_impl(
    config: GenerateChapterConfig,
    default_suffix: &str,
  ) -> Result<CallToolResult, McpError> {
    let new_file_path = Self::generate_new_filename(&config.file_path, config.new_file_name.as_deref(), default_suffix);

    execute_markdown_operation(
      &config.file_path,
      |content| {
        let parser = MarkdownParser::new().map_err(|e| format!("创建解析器失败: {}", e))?;

        let mut mst = parser.parse(content).map_err(|e| format!("解析 Markdown 失败: {}", e))?;

        let numbering_config = NumberingConfig {
          ignore_h1: config.ignore_h1,
          use_chinese_number: config.use_chinese_number,
          use_arabic_number_for_sublevel: config.use_arabic_number_for_sublevel,
        };

        let generator = NumberingGenerator::new(numbering_config);
        generator.generate_numbering(&mut mst);

        let renderer = MarkdownRenderer::new();
        let result = renderer.render_with_numbering(&mst);

        Ok(result)
      },
      format!("成功为文件 {} 生成章节编号", config.file_path),
      config.save_as_new_file,
      &new_file_path,
    )
  }

  pub async fn remove_all_chapter_numbers_impl(
    config: RemoveChapterConfig,
    default_suffix: &str,
  ) -> Result<CallToolResult, McpError> {
    let new_file_path = Self::generate_new_filename(&config.file_path, config.new_file_name.as_deref(), default_suffix);

    execute_markdown_operation(
      &config.file_path,
      |content| {
        let parser = MarkdownParser::new().map_err(|e| format!("创建解析器失败: {}", e))?;

        let mst = parser.parse(content).map_err(|e| format!("解析 Markdown 失败: {}", e))?;

        let renderer = MarkdownRenderer::new();
        let result = renderer.render_without_numbering(&mst);

        Ok(result)
      },
      format!("成功清除文件 {} 的所有章节编号", config.file_path),
      config.save_as_new_file,
      &new_file_path,
    )
  }

  pub async fn check_heading_impl(config: CheckHeadingConfig) -> Result<CallToolResult, McpError> {
    let result = (|| -> crate::error::Result<CallToolResult> {
      crate::utils::validate_markdown_file(&config.file_path)?;

      let content = crate::utils::read_file_content(&config.file_path)?;

      // 解析文档
      let parser =
        MarkdownParser::new().map_err(|e| crate::error::MarkdownError::ParseError(format!("创建解析器失败: {}", e)))?;

      let mst = parser
        .parse(&content)
        .map_err(|e| crate::error::MarkdownError::ParseError(format!("解析 Markdown 失败: {}", e)))?;

      // 验证标题结构
      let validation_result = Self::validate_heading_structure(&mst);

      match validation_result {
        Ok(report) => Ok(CallToolResult::success(vec![Content::text(format!("✅ 标题验证通过\n\n{}", report))])),
        Err(errors) => {
          Ok(CallToolResult::error(vec![Content::text(format!("❌ 标题验证失败\n\n{}", errors.join("\n")))]))
        }
      }
    })();

    result.map_err(|e| e.into())
  }

  /// 验证标题结构
  fn validate_heading_structure(mst: &crate::mst::MSTNode) -> Result<String, Vec<String>> {
    let headers = mst.get_headers();
    let mut errors = Vec::new();
    let mut report_lines = Vec::new();

    if headers.is_empty() {
      return Ok("文档中没有标题行。".to_string());
    }

    // 统计信息
    let mut level_counts = std::collections::HashMap::new();
    for header in &headers {
      if let Some(level) = header.header_level() {
        *level_counts.entry(level).or_insert(0) += 1;
      }
    }

    report_lines.push(format!("📊 标题统计："));
    for level in 1..=6 {
      if let Some(count) = level_counts.get(&level) {
        report_lines.push(format!("  H{}: {} 个", level, count));
      }
    }
    report_lines.push(String::new());

    // 验证每个标题的格式和层级
    let mut prev_level = None;
    let mut level_stack = Vec::new(); // 记录层级栈

    for (i, header) in headers.iter().enumerate() {
      let line_number = header.line_number;
      let raw_line = &header.raw_line;
      let current_level = header.header_level().unwrap();

      // 验证格式
      if let Err(format_error) = Self::validate_heading_format(raw_line, current_level, line_number) {
        errors.push(format_error);
        continue;
      }

      // 验证层级结构
      if let Some(prev) = prev_level {
        // 更新层级栈
        while let Some(&stack_level) = level_stack.last() {
          if stack_level >= current_level {
            level_stack.pop();
          } else {
            break;
          }
        }

        // 检查是否跳级
        if current_level > prev + 1 && level_stack.is_empty() {
          errors.push(format!(
            "第{}行：标题级别跳级，从 H{} 直接跳到 H{}（跳过了 H{}）",
            line_number,
            prev,
            current_level,
            prev + 1
          ));
        } else if current_level > prev + 1 {
          // 检查是否相对于栈顶跳级
          if let Some(&stack_top) = level_stack.last() {
            if current_level > stack_top + 1 {
              errors.push(format!(
                "第{}行：标题级别跳级，从 H{} 直接跳到 H{}（跳过了 H{}）",
                line_number,
                stack_top,
                current_level,
                stack_top + 1
              ));
            }
          }
        }
      }

      level_stack.push(current_level);
      prev_level = Some(current_level);
    }

    if errors.is_empty() {
      report_lines.push("✅ 所有标题格式和层级结构都正确。".to_string());
      Ok(report_lines.join("\n"))
    } else {
      Err(errors)
    }
  }

  /// 验证单个标题的格式
  fn validate_heading_format(raw_line: &str, expected_level: usize, line_number: usize) -> Result<(), String> {
    // 检查是否以正确数量的#开头
    let expected_prefix = "#".repeat(expected_level);

    if !raw_line.starts_with(&expected_prefix) {
      return Err(format!("第{}行：标题格式错误，应该以 {} 开头", line_number, expected_prefix));
    }

    // 检查#前面是否有空格
    if raw_line.chars().next() != Some('#') {
      return Err(format!("第{}行：标题格式错误，# 符号前不能有空格或其他字符", line_number));
    }

    // 检查#后面是否有且仅有一个空格
    let after_hashes = &raw_line[expected_level..];
    if !after_hashes.starts_with(' ') {
      return Err(format!("第{}行：标题格式错误，{} 后面必须有一个空格", line_number, expected_prefix));
    }

    if after_hashes.starts_with("  ") {
      return Err(format!("第{}行：标题格式错误，{} 后面只能有一个空格", line_number, expected_prefix));
    }

    // 检查是否有标题内容
    let title_content = after_hashes.trim_start();
    if title_content.is_empty() {
      return Err(format!("第{}行：标题格式错误，缺少标题内容", line_number));
    }

    Ok(())
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
