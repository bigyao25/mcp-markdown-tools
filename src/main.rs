use rmcp::{
  model::*,
  service::{RequestContext, RoleServer, ServiceExt},
  transport::io::stdio,
  ErrorData as McpError, ServerHandler,
};

mod config;
mod error;
mod image_localizer;
mod mst;
mod numbering;
mod parser;
mod renderer;
mod tools;
mod utils;

use config::{CheckHeadingConfig, GenerateChapterConfig, LocalizeImagesConfig, RemoveChapterConfig};
use tools::MarkdownToolsImpl;

#[derive(Clone)]
pub struct MarkdownTools;

impl ServerHandler for MarkdownTools {
  fn get_info(&self) -> ServerInfo {
    ServerInfo {
      server_info: Implementation { name: "mcp-markdown-tools".to_string(), version: "0.1.0".to_string() },
      instructions: Some("一个 markdown 文档工具集".into()),
      ..Default::default()
    }
  }

  async fn list_tools(
    &self,
    _request: Option<PaginatedRequestParam>,
    _context: RequestContext<RoleServer>,
  ) -> Result<ListToolsResult, McpError> {
    let tools = vec![
            Tool::new(
                "check_heading",
                r#"验证 Markdown 文档标题行的格式规范性和层级结构的正确性。

验证规则：
1. 标题格式规范：
   - 标题行必须以1-6个#符号开头（称为标题符/Heading token）
   - #符号前不能有空格或其他字符
   - #符号后必须有且仅有一个空格，然后是标题内容
   - #符号的数量决定标题级别（1-6级）

2. 层级结构规范：
   - 允许文档开头有非标题内容（如前言、说明等）
   - 文档中第一个标题的级别决定了文档的起始级别
   - 标题级别必须连续，不允许跳级：
     * 允许：同级标题（H2→H2）、下一级标题（H2→H3）、回到任意上级标题（H3→H2、H3→H1）
     * 不允许：跳级（H1→H3、H2→H4等，即跳过中间级别）

3. 验证范围：
   - 只验证标题行（以#开头的行），忽略其他内容行
   - 支持文档开头有非标题内容

返回结果：
- 验证通过：返回成功信息和标题结构统计
- 验证失败：返回详细的错误报告，包括：
  * 错误类型（格式错误/层级跳级）
  * 具体错误行号和内容"#,
                std::sync::Arc::new(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "full_file_path": {
                                "type": "string",
                                "description": "Markdown 文档的文件路径，必须使用绝对路径"
                            }
                        },
                        "required": ["full_file_path"]
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            ),
            Tool::new(
                "generate_chapter_number",
                r#"为 Markdown 文档所有的标题行(Head line)创建编号。
在创建之前，会将全文档的标题行检查一遍，清理掉已有的编号。
为了提高处理速度，你应该直接对整个文件执行该工具，而不是对原文件分段读取处理。"#,
                std::sync::Arc::new(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "full_file_path": {
                                "type": "string",
                                "description": "Markdown 文档的文件路径，必须使用绝对路径"
                            },
                            "ignore_h1": {
                                "type": "boolean",
                                "description": "是否忽略一级标题（# 标题）",
                                "default": false
                            },
                            "use_chinese_number": {
                                "type": "boolean",
                                "description": "是否使用中文编号（一、二、三...）",
                                "default": false
                            },
                            "use_arabic_number_for_sublevel": {
                                "type": "boolean",
                                "description": r#"一级以下编号是否使用独立的阿拉伯数字编号。use_chinese_number=true 时生效。
为 true 时，只有一级编号使用中文，每个一级编号下的二级编号都是用从1开始的一级阿拉伯数字编号，三级编号都是用从1.1开始的二级阿拉伯数字编号，以此类推。

示例一（ignore_h1=false, use_chinese_number=true, use_arabic_number_for_sublevel=true）：
# 一、
## 一、一、
### 一、一、一、
### 一、一、二、
## 一、二、
# 二、
## 二、一、
## 二、二、
# 三、
## 三、一、

示例二（ignore_h1=true, use_chinese_number=true, use_arabic_number_for_sublevel=false）：
## 一、
### 1.
#### 1.1.
#### 1.2.
### 2.
## 二、
### 1.
### 2.
## 三、
### 1.
"#,
                                "default": true
                            },
                            "save_as_new_file": {
                                "type": "boolean",
                                "description": "编辑后，是否另存为新文件，为false时将覆盖原文件。",
                                "default": false
                            },
                            "new_full_file_path": {
                                "type": "string",
                                "description": r#"新文件名，必须使用绝对路径。save_as_new_file=true 时生效。
默认与原文档同目录，默认文件名为：{original_file_name}_numed.md。"#,
                                "default": "{full_dir_of_original_file}/{original_file_name}_numed.md"
                            }
                        },
                        "required": ["full_file_path"]
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            ),
            Tool::new(
                "remove_all_chapter_numbers",
                r#"清除 Markdown 文档所有标题行(Head line)的编号，包括数字和中文编号。
为了提高处理速度，你应该直接对整个文件执行该工具，而不是对原文件分段读取处理。"#,
                std::sync::Arc::new(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "full_file_path": {
                                "type": "string",
                                "description": "Markdown 文档的文件路径，必须使用绝对路径"
                            },
                            "save_as_new_file": {
                                "type": "boolean",
                                "description": "编辑后，是否另存为新文件，文件名为：{原文件名}_unnumed.md。为false时将覆盖原文件。",
                                "default": false
                            },
                            "new_full_file_path": {
                                "type": "string",
                                "description": r#"新文件名，必须使用绝对路径。save_as_new_file=true 时生效。
默认与原文档同目录，默认文件名为：{original_file_name}_unnumed.md。"#,
                                "default": "{full_dir_of_original_file}/{original_file_name}_unnumed.md"
                            }
                        },
                        "required": ["full_file_path"]
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            ),
            Tool::new(
                "localize_images",
                r#"将整个 Markdown 文档中引用的远程图片资源保存到本地，并且更改文档中的引用。
为了提高处理速度，你应该直接对整个文件执行该工具，而不是对原文件分段读取处理。"#,
                std::sync::Arc::new(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "full_file_path": {
                                "type": "string",
                                "description": "Markdown 文档的文件路径，必须使用绝对路径"
                            },
                            "image_file_name_pattern": {
                                "type": "string",
                                "description": "保存到本地的图片文件名格式，不包含扩展名。
支持的通配符有：
- multilevel_num: 图片所在的多层级编号，例如：1.2.1.
- index: 序号，从零开始
- hash: 6位哈希字符",
                                "default": "{multilevel_num}-{index}"
                            },
                            "image_dir": {
                                "type": "string",
                                "description": "图片保存的目录，默认为原文档同目录下的 assets 目录",
                                "default": "{full_dir_of_original_file}/assets/"
                            },
                            "new_full_file_path": {
                                "type": "string",
                                "description": r#"新文件名，必须使用绝对路径。为空则覆盖原文件。"#,
                                "default": ""
                            }
                        },
                        "required": ["full_file_path"]
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            ),
        ];

    Ok(ListToolsResult { next_cursor: None, tools })
  }

  async fn call_tool(
    &self,
    request: CallToolRequestParam,
    _context: RequestContext<RoleServer>,
  ) -> Result<CallToolResult, McpError> {
    match request.name.as_ref() {
      "check_heading" => {
        let config = CheckHeadingConfig::from_args(request.arguments.as_ref())?;
        MarkdownToolsImpl::check_heading_impl(config).await
      }
      "generate_chapter_number" => {
        let config = GenerateChapterConfig::from_args(request.arguments.as_ref())?;
        MarkdownToolsImpl::generate_chapter_number_impl(config, "numed").await
      }
      "remove_all_chapter_numbers" => {
        let config = RemoveChapterConfig::from_args(request.arguments.as_ref())?;
        MarkdownToolsImpl::remove_all_chapter_numbers_impl(config, "unnumed").await
      }
      "localize_images" => {
        let config = LocalizeImagesConfig::from_args(request.arguments.as_ref())?;
        MarkdownToolsImpl::localize_images_impl(config).await
      }
      _ => Err(McpError::method_not_found::<CallToolRequestMethod>()),
    }
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let server = MarkdownTools;
  let transport = stdio();

  eprintln!("Markdown tools MCP Server starting...");

  // 使用ServiceExt trait的serve方法
  let running_service = server.serve(transport).await?;

  eprintln!("Server started successfully");
  running_service.waiting().await?;

  Ok(())
}
