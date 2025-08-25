use crate::error::{MarkdownError, Result};
use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub struct GenerateChapterConfig {
  pub file_path: String,
  pub ignore_h1: bool,
  pub use_chinese_number: bool,
  pub use_arabic_number_for_sublevel: bool,
  pub save_as_new_file: bool,
  pub new_file_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RemoveChapterConfig {
  pub file_path: String,
  pub save_as_new_file: bool,
  pub new_file_name: Option<String>,
}

impl GenerateChapterConfig {
  pub fn from_args(args: Option<&Map<String, Value>>) -> Result<Self> {
    let args = args.ok_or_else(|| MarkdownError::ConfigError("缺少参数".to_string()))?;

    let file_path = args
      .get("file_path")
      .and_then(|v| v.as_str())
      .ok_or_else(|| MarkdownError::ConfigError("缺少 file_path 参数".to_string()))?
      .to_string();

    let ignore_h1 = args.get("ignore_h1").and_then(|v| v.as_bool()).unwrap_or(false);

    let use_chinese_number = args.get("use_chinese_number").and_then(|v| v.as_bool()).unwrap_or(false);

    let use_arabic_number_for_sublevel =
      args.get("use_arabic_number_for_sublevel").and_then(|v| v.as_bool()).unwrap_or(true);

    let save_as_new_file = args.get("save_as_new_file").and_then(|v| v.as_bool()).unwrap_or(false);

    let new_file_name = args.get("new_file_name").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(Self {
      file_path,
      ignore_h1,
      use_chinese_number,
      use_arabic_number_for_sublevel,
      save_as_new_file,
      new_file_name,
    })
  }
}

impl RemoveChapterConfig {
  pub fn from_args(args: Option<&Map<String, Value>>) -> Result<Self> {
    let args = args.ok_or_else(|| MarkdownError::ConfigError("缺少参数".to_string()))?;

    let file_path = args
      .get("file_path")
      .and_then(|v| v.as_str())
      .ok_or_else(|| MarkdownError::ConfigError("缺少 file_path 参数".to_string()))?
      .to_string();

    let save_as_new_file = args.get("save_as_new_file").and_then(|v| v.as_bool()).unwrap_or(false);

    let new_file_name = args.get("new_file_name").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(Self { file_path, save_as_new_file, new_file_name })
  }
}
