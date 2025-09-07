use std::path::Path;

use crate::config::{CheckHeadingConfig, GenerateChapterConfig, LocalizeImagesConfig, RemoveChapterConfig};
use crate::image_localizer::ImageLocalizer;
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
    let new_full_file_path =
      Self::generate_new_filename(&config.full_file_path, config.new_full_file_path.as_deref(), default_suffix);

    execute_markdown_operation(
      &config.full_file_path,
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
      format!("成功为文件 {} 生成章节编号", config.full_file_path),
      config.save_as_new_file,
      new_full_file_path.as_str(),
    )
  }

  pub async fn remove_all_chapter_numbers_impl(
    config: RemoveChapterConfig,
    default_suffix: &str,
  ) -> Result<CallToolResult, McpError> {
    let new_full_file_path =
      Self::generate_new_filename(&config.full_file_path, config.new_full_file_path.as_deref(), default_suffix);

    execute_markdown_operation(
      &config.full_file_path,
      |content| {
        let parser = MarkdownParser::new().map_err(|e| format!("创建解析器失败: {}", e))?;

        let mst = parser.parse(content).map_err(|e| format!("解析 Markdown 失败: {}", e))?;

        let renderer = MarkdownRenderer::new();
        let result = renderer.render_without_numbering(&mst);

        Ok(result)
      },
      format!("成功清除文件 {} 的所有章节编号", config.full_file_path),
      config.save_as_new_file,
      new_full_file_path.as_str(),
    )
  }

  pub async fn check_heading_impl(config: CheckHeadingConfig) -> Result<CallToolResult, McpError> {
    let result = (|| -> crate::error::Result<CallToolResult> {
      crate::utils::validate_markdown_file(&config.full_file_path)?;

      let content = crate::utils::read_file_content(&config.full_file_path)?;

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

    for (_i, header) in headers.iter().enumerate() {
      let line_number = header.line_number;
      let raw = &header.raw;
      let current_level = header.header_level().unwrap();

      // 验证格式
      if let Err(format_error) = Self::validate_heading_format(raw, current_level, line_number) {
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
  fn validate_heading_format(raw: &str, expected_level: usize, line_number: usize) -> Result<(), String> {
    // 检查是否以正确数量的#开头
    let expected_prefix = "#".repeat(expected_level);

    if !raw.starts_with(&expected_prefix) {
      return Err(format!("第{}行：标题格式错误，应该以 {} 开头", line_number, expected_prefix));
    }

    // 检查#前面是否有空格
    if raw.chars().next() != Some('#') {
      return Err(format!("第{}行：标题格式错误，# 符号前不能有空格或其他字符", line_number));
    }

    // 检查#后面是否有且仅有一个空格
    let after_hashes = &raw[expected_level..];
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
  fn generate_new_filename(full_file_path: &str, new_full_file_path: Option<&str>, default_suffix: &str) -> String {
    let path = Path::new(full_file_path);
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem().unwrap_or_else(|| std::ffi::OsStr::new("file")).to_str().unwrap_or("file");
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("md");

    let new_path = match new_full_file_path {
      Some(name) => Path::new(name).to_path_buf(),
      None => parent.join(format!("{}_{}.{}", stem, default_suffix, extension)),
    };
    new_path.to_str().unwrap().to_string()
  }

  /// 本地化图片实现
  pub async fn localize_images_impl(config: LocalizeImagesConfig) -> Result<CallToolResult, McpError> {
    // 验证文件
    if let Err(e) = crate::utils::validate_markdown_file(&config.full_file_path) {
      return Ok(CallToolResult::error(vec![Content::text(format!("文件验证失败: {}", e))]));
    }

    // 读取文件内容
    let content = match crate::utils::read_file_content(&config.full_file_path) {
      Ok(content) => content,
      Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("读取文件失败: {}", e))])),
    };

    // 解析文档
    let parser = match MarkdownParser::new() {
      Ok(parser) => parser,
      Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("创建解析器失败: {}", e))])),
    };

    let mut mst = match parser.parse(&content) {
      Ok(mst) => mst,
      Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("解析 Markdown 失败: {}", e))])),
    };

    // 创建图片本地化器
    let localizer = ImageLocalizer::new(config.clone());

    // 本地化图片
    let results = match localizer.localize_images(&mut mst).await {
      Ok(results) => results,
      Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("图片本地化失败: {}", e))])),
    };

    // 渲染更新后的文档
    let renderer = MarkdownRenderer::new();
    let new_content = renderer.render(&mst);

    // 写回文件
    let save_full_file_path = match config.new_full_file_path {
      Some(p) => p,
      None => config.full_file_path.clone(),
    };
    if let Err(e) = crate::utils::write_file_content(&save_full_file_path, &new_content) {
      return Ok(CallToolResult::error(vec![Content::text(format!("写入文件失败: {}", e))]));
    }

    // 生成结果报告
    let mut report = vec![format!("✅ 处理完毕: {}", config.full_file_path)];
    report.extend(results);

    Ok(CallToolResult::success(vec![Content::text(report.join("\n"))]))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::{NamedTempFile, TempDir};

  /// 测试生成章节编号 - 阿拉伯数字
  #[tokio::test]
  async fn test_generate_chapter_number_arabic() {
    let content = r#"# 第一章

## 背景

### 历史

## 目标

# 第二章

## 实现
"#;

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));

    // 验证文件内容被修改
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(new_content.contains("# 1. 第一章"));
    assert!(new_content.contains("## 1.1. 背景"));
    assert!(new_content.contains("### 1.1.1. 历史"));
    assert!(new_content.contains("## 1.2. 目标"));
    assert!(new_content.contains("# 2. 第二章"));
    assert!(new_content.contains("## 2.1. 实现"));
  }

  /// 测试生成章节编号 - 中文数字
  #[tokio::test]
  async fn test_generate_chapter_number_chinese() {
    let content = r#"# 第一章

## 背景

# 第二章

## 实现
"#;

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: true,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));

    // 验证文件内容被修改
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(new_content.contains("# 一、第一章"));
    assert!(new_content.contains("## 1. 背景"));
    assert!(new_content.contains("# 二、第二章"));
    assert!(new_content.contains("## 1. 实现"));
  }

  /// 测试生成章节编号 - 忽略 H1
  #[tokio::test]
  async fn test_generate_chapter_number_ignore_h1() {
    let content = r#"# 文档标题

## 第一章

### 背景

## 第二章

### 实现
"#;

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: true,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));

    // 验证文件内容被修改
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(new_content.contains("# 文档标题")); // H1 不变
    assert!(new_content.contains("## 1. 第一章"));
    assert!(new_content.contains("### 1.1. 背景"));
    assert!(new_content.contains("## 2. 第二章"));
    assert!(new_content.contains("### 2.1. 实现"));
  }

  /// 测试生成章节编号 - 保存为新文件
  #[tokio::test]
  async fn test_generate_chapter_number_save_as_new() {
    let content = r#"# 第一章

## 背景
"#;

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let new_file_path = temp_dir.path().join("new_file.md");

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: true,
      new_full_file_path: Some(new_file_path.to_str().unwrap().to_string()),
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));

    // 验证原文件未被修改
    let original_content = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(original_content, content);

    // 验证新文件被创建并包含编号
    assert!(new_file_path.exists());
    let new_content = fs::read_to_string(&new_file_path).unwrap();
    assert!(new_content.contains("# 1. 第一章"));
    assert!(new_content.contains("## 1.1. 背景"));
  }

  /// 测试移除章节编号
  #[tokio::test]
  async fn test_remove_all_chapter_numbers() {
    let content = r#"# 1. 第一章

## 1.1. 背景

### 1.1.1. 历史

## 1.2. 目标

# 2. 第二章

## 2.1. 实现
"#;

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = RemoveChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::remove_all_chapter_numbers_impl(config, "unnumed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));

    // 验证编号被移除
    let new_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(new_content.contains("# 第一章"));
    assert!(new_content.contains("## 背景"));
    assert!(new_content.contains("### 历史"));
    assert!(new_content.contains("## 目标"));
    assert!(new_content.contains("# 第二章"));
    assert!(new_content.contains("## 实现"));

    // 确保数字编号被完全移除
    assert!(!new_content.contains("1."));
    assert!(!new_content.contains("2."));
  }

  /// 测试检查标题 - 有效标题
  #[tokio::test]
  async fn test_check_heading_valid() {
    let content = r#"# 第一章

## 1.1 背景

### 1.1.1 历史

## 1.2 目标

# 第二章

## 2.1 实现
"#;

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = CheckHeadingConfig { full_file_path: temp_file.path().to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));
  }

  /// 测试检查标题 - 无效标题格式
  #[tokio::test]
  async fn test_check_heading_invalid_format() {
    let content = r#"# 正确的标题

##错误的标题

### 正确的三级标题

####  错误的标题
"#;

    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), content).unwrap();

    let config = CheckHeadingConfig { full_file_path: temp_file.path().to_str().unwrap().to_string() };

    let result = MarkdownToolsImpl::check_heading_impl(config).await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));
  }

  /// 测试文件验证错误
  #[tokio::test]
  async fn test_file_validation_error() {
    let config = GenerateChapterConfig {
      full_file_path: "/nonexistent/file.md".to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_err());
  }

  /// 测试空文档处理
  #[tokio::test]
  async fn test_empty_document() {
    let temp_file = NamedTempFile::with_suffix(".md").unwrap();
    fs::write(temp_file.path(), "").unwrap();

    let config = GenerateChapterConfig {
      full_file_path: temp_file.path().to_str().unwrap().to_string(),
      ignore_h1: false,
      use_chinese_number: false,
      use_arabic_number_for_sublevel: true,
      save_as_new_file: false,
      new_full_file_path: None,
    };

    let result = MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await;

    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(false));

    // 空文档应该保持为空
    let content = fs::read_to_string(temp_file.path()).unwrap();
    assert_eq!(content, "");
  }
}
