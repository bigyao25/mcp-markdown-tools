use crate::error::{MarkdownError, Result};
use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub struct GenerateChapterConfig {
  pub full_file_path: String,
  pub ignore_h1: bool,
  pub use_chinese_number: bool,
  pub use_arabic_number_for_sublevel: bool,
  pub save_as_new_file: bool,
  pub new_full_file_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RemoveChapterConfig {
  pub full_file_path: String,
  pub save_as_new_file: bool,
  pub new_full_file_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CheckHeadingConfig {
  pub full_file_path: String,
}

#[derive(Debug, Clone)]
pub struct LocalizeImagesConfig {
  pub full_file_path: String,
  pub image_file_name_pattern: String,
  pub save_to_dir: String,
}

impl GenerateChapterConfig {
  pub fn from_args(args: Option<&Map<String, Value>>) -> Result<Self> {
    let args = args.ok_or_else(|| MarkdownError::ConfigError("缺少参数".to_string()))?;

    let full_file_path = args
      .get("full_file_path")
      .and_then(|v| v.as_str())
      .ok_or_else(|| MarkdownError::ConfigError("缺少 full_file_path 参数".to_string()))?
      .to_string();

    let ignore_h1 = args.get("ignore_h1").and_then(|v| v.as_bool()).unwrap_or(false);

    let use_chinese_number = args.get("use_chinese_number").and_then(|v| v.as_bool()).unwrap_or(false);

    let use_arabic_number_for_sublevel =
      args.get("use_arabic_number_for_sublevel").and_then(|v| v.as_bool()).unwrap_or(true);

    let save_as_new_file = args.get("save_as_new_file").and_then(|v| v.as_bool()).unwrap_or(false);

    let new_full_file_path = args.get("new_full_file_path").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(Self {
      full_file_path,
      ignore_h1,
      use_chinese_number,
      use_arabic_number_for_sublevel,
      save_as_new_file,
      new_full_file_path,
    })
  }
}

impl RemoveChapterConfig {
  pub fn from_args(args: Option<&Map<String, Value>>) -> Result<Self> {
    let args = args.ok_or_else(|| MarkdownError::ConfigError("缺少参数".to_string()))?;

    let full_file_path = args
      .get("full_file_path")
      .and_then(|v| v.as_str())
      .ok_or_else(|| MarkdownError::ConfigError("缺少 full_file_path 参数".to_string()))?
      .to_string();

    let save_as_new_file = args.get("save_as_new_file").and_then(|v| v.as_bool()).unwrap_or(false);

    let new_full_file_path = args.get("new_full_file_path").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(Self { full_file_path, save_as_new_file, new_full_file_path })
  }
}

impl CheckHeadingConfig {
  pub fn from_args(args: Option<&Map<String, Value>>) -> Result<Self> {
    let args = args.ok_or_else(|| MarkdownError::ConfigError("缺少参数".to_string()))?;

    let full_file_path = args
      .get("full_file_path")
      .and_then(|v| v.as_str())
      .ok_or_else(|| MarkdownError::ConfigError("缺少 full_file_path 参数".to_string()))?
      .to_string();

    Ok(Self { full_file_path })
  }
}

impl LocalizeImagesConfig {
  pub fn from_args(args: Option<&Map<String, Value>>) -> Result<Self> {
    let args = args.ok_or_else(|| MarkdownError::ConfigError("缺少参数".to_string()))?;

    let full_file_path = args
      .get("full_file_path")
      .and_then(|v| v.as_str())
      .ok_or_else(|| MarkdownError::ConfigError("缺少 full_file_path 参数".to_string()))?
      .to_string();

    let image_file_name_pattern =
      args.get("image_file_name_pattern").and_then(|v| v.as_str()).unwrap_or("{multilevel_num}-{index}").to_string();

    let save_to_dir =
      args.get("save_to_dir").and_then(|v| v.as_str()).unwrap_or("{full_dir_of_original_file}/assets/").to_string();

    Ok(Self { full_file_path, image_file_name_pattern, save_to_dir })
  }

  /// 获取处理占位符后的保存目录
  pub fn get_resolved_save_dir(&self) -> String {
    use std::path::Path;

    if self.save_to_dir.contains("{full_dir_of_original_file}") {
      let path = Path::new(&self.full_file_path);
      let full_dir = path.parent().unwrap_or(Path::new(".")).to_str().unwrap_or(".");
      self.save_to_dir.replace("{full_dir_of_original_file}", full_dir)
    } else {
      self.save_to_dir.clone()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::{Map, Value};

  /// 测试 GenerateChapterConfig 的有效参数解析
  #[test]
  fn test_generate_chapter_config_from_valid_args() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("/path/to/file.md".to_string()));
    args.insert("ignore_h1".to_string(), Value::Bool(true));
    args.insert("use_chinese_number".to_string(), Value::Bool(true));
    args.insert("use_arabic_number_for_sublevel".to_string(), Value::Bool(false));
    args.insert("save_as_new_file".to_string(), Value::Bool(true));
    args.insert("new_full_file_path".to_string(), Value::String("/path/to/new_file.md".to_string()));

    let config = GenerateChapterConfig::from_args(Some(&args)).unwrap();

    assert_eq!(config.full_file_path, "/path/to/file.md");
    assert_eq!(config.ignore_h1, true);
    assert_eq!(config.use_chinese_number, true);
    assert_eq!(config.use_arabic_number_for_sublevel, false);
    assert_eq!(config.save_as_new_file, true);
    assert_eq!(config.new_full_file_path, Some("/path/to/new_file.md".to_string()));
  }

  /// 测试 GenerateChapterConfig 的默认值
  #[test]
  fn test_generate_chapter_config_defaults() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("/path/to/file.md".to_string()));

    let config = GenerateChapterConfig::from_args(Some(&args)).unwrap();

    assert_eq!(config.full_file_path, "/path/to/file.md");
    assert_eq!(config.ignore_h1, false);
    assert_eq!(config.use_chinese_number, false);
    assert_eq!(config.use_arabic_number_for_sublevel, true);
    assert_eq!(config.save_as_new_file, false);
    assert_eq!(config.new_full_file_path, None);
  }

  /// 测试 GenerateChapterConfig 缺少必需参数的错误
  #[test]
  fn test_generate_chapter_config_missing_required_param() {
    let args = Map::new();
    let result = GenerateChapterConfig::from_args(Some(&args));

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("缺少 full_file_path 参数"));
  }

  /// 测试 GenerateChapterConfig 参数为 None 的错误
  #[test]
  fn test_generate_chapter_config_none_args() {
    let result = GenerateChapterConfig::from_args(None);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("缺少参数"));
  }

  /// 测试 GenerateChapterConfig 参数类型错误
  #[test]
  fn test_generate_chapter_config_wrong_type() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::Number(123.into()));

    let result = GenerateChapterConfig::from_args(Some(&args));

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("缺少 full_file_path 参数"));
  }

  /// 测试 RemoveChapterConfig 的有效参数解析
  #[test]
  fn test_remove_chapter_config_from_valid_args() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("/path/to/file.md".to_string()));
    args.insert("save_as_new_file".to_string(), Value::Bool(true));
    args.insert("new_full_file_path".to_string(), Value::String("/path/to/new_file.md".to_string()));

    let config = RemoveChapterConfig::from_args(Some(&args)).unwrap();

    assert_eq!(config.full_file_path, "/path/to/file.md");
    assert_eq!(config.save_as_new_file, true);
    assert_eq!(config.new_full_file_path, Some("/path/to/new_file.md".to_string()));
  }

  /// 测试 RemoveChapterConfig 的默认值
  #[test]
  fn test_remove_chapter_config_defaults() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("/path/to/file.md".to_string()));

    let config = RemoveChapterConfig::from_args(Some(&args)).unwrap();

    assert_eq!(config.full_file_path, "/path/to/file.md");
    assert_eq!(config.save_as_new_file, false);
    assert_eq!(config.new_full_file_path, None);
  }

  /// 测试 CheckHeadingConfig 的有效参数解析
  #[test]
  fn test_check_heading_config_from_valid_args() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("/path/to/file.md".to_string()));

    let config = CheckHeadingConfig::from_args(Some(&args)).unwrap();

    assert_eq!(config.full_file_path, "/path/to/file.md");
  }

  /// 测试 LocalizeImagesConfig 的有效参数解析
  #[test]
  fn test_localize_images_config_from_valid_args() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("/path/to/file.md".to_string()));
    args.insert("image_file_name_pattern".to_string(), Value::String("{index}-{hash}".to_string()));
    args.insert("save_to_dir".to_string(), Value::String("/path/to/images/".to_string()));

    let config = LocalizeImagesConfig::from_args(Some(&args)).unwrap();

    assert_eq!(config.full_file_path, "/path/to/file.md");
    assert_eq!(config.image_file_name_pattern, "{index}-{hash}");
    assert_eq!(config.save_to_dir, "/path/to/images/");
  }

  /// 测试 LocalizeImagesConfig 的默认值
  #[test]
  fn test_localize_images_config_defaults() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("/path/to/file.md".to_string()));

    let config = LocalizeImagesConfig::from_args(Some(&args)).unwrap();

    assert_eq!(config.full_file_path, "/path/to/file.md");
    assert_eq!(config.image_file_name_pattern, "{multilevel_num}-{index}");
    assert_eq!(config.save_to_dir, "{full_dir_of_original_file}/assets/");
  }

  /// 测试 LocalizeImagesConfig 的 get_resolved_save_dir 方法
  #[test]
  fn test_localize_images_config_get_resolved_save_dir() {
    // 测试包含占位符的情况
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("/home/user/docs/test.md".to_string()));
    args.insert("save_to_dir".to_string(), Value::String("{full_dir_of_original_file}/assets/".to_string()));

    let config = LocalizeImagesConfig::from_args(Some(&args)).unwrap();
    let resolved_dir = config.get_resolved_save_dir();

    assert_eq!(resolved_dir, "/home/user/docs/assets/");
  }

  /// 测试 LocalizeImagesConfig 的 get_resolved_save_dir 方法（无占位符）
  #[test]
  fn test_localize_images_config_get_resolved_save_dir_no_placeholder() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("/home/user/docs/test.md".to_string()));
    args.insert("save_to_dir".to_string(), Value::String("/absolute/path/images/".to_string()));

    let config = LocalizeImagesConfig::from_args(Some(&args)).unwrap();
    let resolved_dir = config.get_resolved_save_dir();

    assert_eq!(resolved_dir, "/absolute/path/images/");
  }

  /// 测试 LocalizeImagesConfig 处理根目录文件的情况
  #[test]
  fn test_localize_images_config_root_file() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("test.md".to_string()));
    args.insert("save_to_dir".to_string(), Value::String("{full_dir_of_original_file}/assets/".to_string()));

    let config = LocalizeImagesConfig::from_args(Some(&args)).unwrap();
    let resolved_dir = config.get_resolved_save_dir();

    assert_eq!(resolved_dir, "/assets/");
  }

  /// 测试配置结构的 Clone 和 Debug 特性
  #[test]
  fn test_config_traits() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("/path/to/file.md".to_string()));

    let config = GenerateChapterConfig::from_args(Some(&args)).unwrap();
    let cloned_config = config.clone();

    assert_eq!(config.full_file_path, cloned_config.full_file_path);

    // 测试 Debug 输出
    let debug_output = format!("{:?}", config);
    assert!(debug_output.contains("GenerateChapterConfig"));
    assert!(debug_output.contains("/path/to/file.md"));
  }

  /// 测试边界情况：空字符串路径
  #[test]
  fn test_empty_file_path() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("".to_string()));

    let config = GenerateChapterConfig::from_args(Some(&args)).unwrap();
    assert_eq!(config.full_file_path, "");
  }

  /// 测试边界情况：特殊字符路径
  #[test]
  fn test_special_characters_in_path() {
    let mut args = Map::new();
    args.insert("full_file_path".to_string(), Value::String("/path/with spaces/中文/file.md".to_string()));

    let config = GenerateChapterConfig::from_args(Some(&args)).unwrap();
    assert_eq!(config.full_file_path, "/path/with spaces/中文/file.md");
  }
}
