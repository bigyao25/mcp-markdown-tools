use crate::utils::execute_markdown_operation;
use regex::Regex;
use rmcp::{ErrorData as McpError, model::*};

pub struct MarkdownToolsImpl;

/// 将阿拉伯数字转换为中文数字
fn to_chinese_number(num: usize) -> String {
    let chinese_digits = ["", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    let _chinese_units = ["", "十", "百", "千"];

    if num == 0 {
        return "零".to_string();
    }

    if num < 10 {
        return chinese_digits[num].to_string();
    }

    if num < 100 {
        let tens = num / 10;
        let ones = num % 10;
        if tens == 1 {
            if ones == 0 {
                return "十".to_string();
            } else {
                return format!("十{}", chinese_digits[ones]);
            }
        } else {
            if ones == 0 {
                return format!("{}十", chinese_digits[tens]);
            } else {
                return format!("{}十{}", chinese_digits[tens], chinese_digits[ones]);
            }
        }
    }

    // 对于更大的数字，简化处理
    num.to_string()
}

/// 生成带编号的内容
fn generate_numbered_content(
    content: &str,
    ignore_h1: bool,
    use_chinese_numbers: bool,
) -> Result<String, String> {
    let header_regex =
        Regex::new(r"^(#{1,6})\s+(.*)$").map_err(|e| format!("正则表达式错误: {}", e))?;

    let mut counters = vec![0; 6]; // 支持6级标题的计数器
    let mut result = Vec::new();

    for line in content.lines() {
        if let Some(captures) = header_regex.captures(line) {
            let hashes = captures.get(1).unwrap().as_str();
            let title = captures.get(2).unwrap().as_str();
            let level = hashes.len();

            // 如果设置忽略一级标题且当前是一级标题，直接添加原行
            if ignore_h1 && level == 1 {
                result.push(line.to_string());
                continue;
            }

            // 更新当前级别的计数器
            counters[level - 1] += 1;

            // 重置子级的计数器
            for i in level..6 {
                counters[i] = 0;
            }

            // 生成编号
            let mut number_parts = Vec::new();
            let start_level = if ignore_h1 { 1 } else { 0 }; // 如果忽略H1，从H2开始编号

            for i in start_level..level {
                if counters[i] > 0 {
                    if use_chinese_numbers {
                        number_parts.push(to_chinese_number(counters[i]));
                    } else {
                        number_parts.push(counters[i].to_string());
                    }
                }
            }

            let number_str = if number_parts.is_empty() {
                String::new()
            } else {
                if use_chinese_numbers {
                    format!("{}、", number_parts.join("、"))
                } else {
                    format!("{}. ", number_parts.join("."))
                }
            };

            result.push(format!("{} {}{}", hashes, number_str, title));
        } else {
            result.push(line.to_string());
        }
    }

    Ok(result.join("\n"))
}

impl MarkdownToolsImpl {
    pub async fn generate_chapter_number_impl(
        file_path: String,
        ignore_h1: bool,
        use_chinese_numbers: bool,
    ) -> Result<CallToolResult, McpError> {
        execute_markdown_operation(
            &file_path,
            |content| generate_numbered_content(content, ignore_h1, use_chinese_numbers),
            format!("成功为文件 {} 生成章节编号", file_path),
        )
    }

    pub async fn insert_tail_impl(file_path: String) -> Result<CallToolResult, McpError> {
        execute_markdown_operation(
            &file_path,
            |content| {
                // 在底部插入新行
                let new_content = if content.ends_with('\n') {
                    format!("{}# Head999\n", content)
                } else {
                    format!("{}\n# Head999\n", content)
                };
                Ok(new_content)
            },
            format!("成功在文件 {} 底部插入【# Head999】", file_path),
        )
    }
}
