use rmcp::{
    ErrorData as McpError, ServerHandler,
    model::*,
    service::{RequestContext, RoleServer, ServiceExt},
    transport::io::stdio,
};
use schemars::JsonSchema;
use serde::Deserialize;

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
        ServerInfo {
            instructions: Some("一个 markdown 文档工具集。如果需要生成新文档，为了加快处理速度，应该先将原文档复制出一份来再进行编辑。".into()),
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
                "generate_chapter_number",
                "为 Markdown 文档所有的标题行(Head line)创建编号",
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
                            "use_chinese_numbers": {
                                "type": "boolean",
                                "description": "是否使用中文编号（一、二、三...）",
                                "default": false
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
                "insert_tail",
                "在Markdown文档底部插入新行，内容为【# Head999】",
                std::sync::Arc::new(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "file_path": {
                                "type": "string",
                                "description": "Markdown文档的文件路径"
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

        Ok(ListToolsResult {
            next_cursor: None,
            tools,
        })
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

                let ignore_h1 = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("ignore_h1"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let use_chinese_numbers = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("use_chinese_numbers"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                MarkdownToolsImpl::generate_chapter_number_impl(
                    file_path.to_string(),
                    ignore_h1,
                    use_chinese_numbers,
                )
                .await
            }
            "insert_tail" => {
                let file_path = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("file_path"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing file_path parameter", None))?;

                MarkdownToolsImpl::insert_tail_impl(file_path.to_string()).await
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
