//! Markdown Tools 库
//!
//! 提供 Markdown 文档处理功能，包括：
//! - 章节编号生成和移除
//! - 标题格式验证
//! - 图片本地化
//! - MST (Markdown Structured Tree) 解析和渲染

pub mod config;
pub mod error;
pub mod image_localizer;
pub mod mst;
pub mod numbering;
pub mod parser;
pub mod renderer;
pub mod tools;
pub mod utils;

// 重新导出常用类型
pub use config::*;
pub use error::{MarkdownError, Result};
pub use mst::{MSTNode, NodeType, NumberingConfig};
pub use tools::MarkdownToolsImpl;
