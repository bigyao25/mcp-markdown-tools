use rmcp::{
  model::*,
  service::{RequestContext, RoleServer, ServiceExt},
  transport::io::stdio,
  ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;

mod mst;
mod numbering;
mod parser;
mod renderer;
mod tools;
mod utils;

use tools::MarkdownToolsImpl;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InsertHeadRequest {
  #[schemars(description = "Markdown文档的文件路径")]
  pub file_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InsertTailRequest {
  #[schemars(description = "Markdown文档的文件路径")]
  pub file_path: String,
}

#[derive(Clone)]
pub struct MarkdownTools;

impl ServerHandler for MarkdownTools {
  fn get_info(&self) -> ServerInfo {
    ServerInfo { instructions: Some("一个 markdown 文档工具集。".into()), ..Default::default() }
  }

  async fn list_tools(
    &self,
    _request: Option<PaginatedRequestParam>,
    _context: RequestContext<RoleServer>,
  ) -> Result<ListToolsResult, McpError> {
    let tools = vec![
            Tool::new(
                "generate_chapter_number",
                r#"为 Markdown 文档所有的标题行(Head line)创建编号。
在创建之前，会将全文档的标题行检查一遍，清理掉已有的编号。
为了提高处理速度，你应该直接对整个文件执行该工具，而不是对原文件分段读取处理。"#,
                std::sync::Arc::new(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "file_path": {
                                "type": "string",
                                "description": "Markdown 文档的文件路径"
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
                            "new_file_name": {
                                "type": "string",
                                "description": "新文件名，不包括扩展名(.md)。save_as_new_file=true 时生效。建议新文件名为：{ori_file_name}_numed.md。",
                                "default": "my_doc_numed"
                            }
                        },
                        "required": ["file_path"]
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
                            "file_path": {
                                "type": "string",
                                "description": "Markdown文档的文件路径"
                            },
                            "save_as_new_file": {
                                "type": "boolean",
                                "description": "编辑后，是否另存为新文件，文件名为：{原文件名}_unnumed.md。为false时将覆盖原文件。",
                                "default": false
                            },
                            "new_file_name": {
                                "type": "string",
                                "description": "新文件名，不包括扩展名(.md)。save_as_new_file=true 时生效。建议新文件名为：{ori_file_name}_unnumed.md。",
                                "default": "my_doc_unnumed"
                            }
                        },
                        "required": ["file_path"]
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
      "generate_chapter_number" => {
        let file_path = request
          .arguments
          .as_ref()
          .and_then(|args| args.get("file_path"))
          .and_then(|v| v.as_str())
          .ok_or_else(|| McpError::invalid_params("Missing file_path parameter", None))?;

        let ignore_h1 =
          request.arguments.as_ref().and_then(|args| args.get("ignore_h1")).and_then(|v| v.as_bool()).unwrap_or(false);

        let use_chinese_number = request
          .arguments
          .as_ref()
          .and_then(|args| args.get("use_chinese_number"))
          .and_then(|v| v.as_bool())
          .unwrap_or(false);

        let use_arabic_number_for_sublevel = request
          .arguments
          .as_ref()
          .and_then(|args| args.get("use_arabic_number_for_sublevel"))
          .and_then(|v| v.as_bool())
          .unwrap_or(true);

        let save_as_new_file = request
          .arguments
          .as_ref()
          .and_then(|args| args.get("save_as_new_file"))
          .and_then(|v| v.as_bool())
          .unwrap_or(false);

        let new_file_name =
          request.arguments.as_ref().and_then(|args| args.get("new_file_name")).and_then(|v| v.as_str());

        MarkdownToolsImpl::generate_chapter_number_impl(
          file_path,
          ignore_h1,
          use_chinese_number,
          use_arabic_number_for_sublevel,
          save_as_new_file,
          new_file_name,
          "numed",
        )
        .await
      }
      "remove_all_chapter_numbers" => {
        let file_path = request
          .arguments
          .as_ref()
          .and_then(|args| args.get("file_path"))
          .and_then(|v| v.as_str())
          .ok_or_else(|| McpError::invalid_params("Missing file_path parameter", None))?;

        let save_as_new_file = request
          .arguments
          .as_ref()
          .and_then(|args| args.get("save_as_new_file"))
          .and_then(|v| v.as_bool())
          .unwrap_or(false);

        let new_file_name =
          request.arguments.as_ref().and_then(|args| args.get("new_file_name")).and_then(|v| v.as_str());

        MarkdownToolsImpl::remove_all_chapter_numbers_impl(file_path, save_as_new_file, new_file_name, "unnumed").await
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
