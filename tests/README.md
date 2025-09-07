# 测试文档

本文档描述了项目的测试组织结构、运行方法和最佳实践。

## 测试架构

### 目录结构

```
tests/
├── unit/                    # 单元测试
│   ├── mod.rs              # 单元测试模块声明
│   ├── numbering_tests.rs  # 编号功能单元测试
│   ├── parser_tests.rs     # 解析器单元测试
│   └── validation_tests.rs # 验证功能单元测试
├── integration/             # 集成测试
│   ├── mod.rs              # 集成测试模块声明
│   ├── numbering_integration_tests.rs
│   └── image_localization_integration_tests.rs
├── e2e/                     # 端到端测试
│   ├── mod.rs              # 端到端测试模块声明
│   └── workflow_tests.rs   # 完整工作流测试
├── common/                  # 公共测试工具
│   ├── mod.rs              # 公共模块声明
│   ├── helpers.rs          # 测试辅助函数
│   └── mock_server.rs      # Mock HTTP 服务器
├── fixtures/                # 测试数据文件
│   ├── simple.md
│   ├── complex.md
│   └── with_images.md
└── README.md               # 本文档
```

### 测试分类

#### 单元测试 (Unit Tests)

- **目的**: 测试单个函数或方法的功能
- **范围**: 独立的代码单元，不依赖外部系统
- **命名规范**: `test_{function_name}_{scenario}`
- **示例**: `test_generate_arabic_numbering_basic()`

#### 集成测试 (Integration Tests)

- **目的**: 测试模块间的交互和协同工作
- **范围**: 多个模块的组合功能
- **命名规范**: `integration_{feature}_{scenario}`
- **示例**: `integration_complete_numbering_workflow()`

#### 端到端测试 (E2E Tests)

- **目的**: 测试完整的用户使用场景
- **范围**: 从用户输入到最终输出的完整流程
- **命名规范**: `e2e_{workflow_name}_{scenario}`
- **示例**: `e2e_complete_document_workflow()`

## 公共测试工具

### TestFileManager

管理测试文件和临时目录：

```rust
let file_manager = TestFileManager::new();
let md_file = file_manager.create_md_file("test.md", content);
let assets_dir = file_manager.assets_dir();
```

### 配置构建器

简化测试配置创建：

```rust
// 编号配置
let config = NumberingConfigBuilder::new(file_path)
    .use_chinese_number(true)
    .ignore_h1(true)
    .build();

// 图片本地化配置
let config = ImageLocalizationConfigBuilder::new(file_path)
    .file_name_pattern("img_{index}_{hash}")
    .image_dir(assets_dir)
    .build();
```

### 断言辅助函数

提供常用的测试断言：

```rust
use crate::common::assertions;

assertions::assert_file_contains(&file_path, "expected content");
assertions::assert_file_not_contains(&file_path, "unexpected content");
assertions::assert_dir_exists(&dir_path);
assertions::assert_file_count(&dir_path, 5);
```

### Mock HTTP 服务器

用于测试网络相关功能：

```rust
#[cfg(feature = "mock")]
let mock_server = MockHttpServer::new().await;
mock_server.mock_multiple_images(10).await;
let base_url = mock_server.url();
```

### 测试数据

预定义的测试内容：

```rust
use crate::common::test_data;

// 使用预定义的测试文档
let content = test_data::SIMPLE_DOC;
let content = test_data::COMPLEX_DOC;
let content = test_data::DOC_WITHOUT_IMAGES;

// 动态生成包含图片的文档
let content = test_data::doc_with_images("https://example.com");
```

## 运行测试

### 运行所有测试

```bash
cargo test
```

### 运行特定类型的测试

```bash
# 单元测试
cargo test --test '' unit

# 集成测试
cargo test --test '' integration

# 端到端测试
cargo test --test '' e2e
```

### 运行特定模块的测试

```bash
# 编号功能测试
cargo test numbering

# 图片本地化测试
cargo test image_localization

# 验证功能测试
cargo test validation
```

### 运行带 Mock 功能的测试

```bash
cargo test --features mock
```

### 显示测试输出

```bash
cargo test -- --nocapture
```

### 并行运行测试

```bash
cargo test -- --test-threads=4
```

## 测试最佳实践

### 1. 测试命名

- 使用描述性的测试名称
- 遵循统一的命名规范
- 包含测试场景的关键信息

### 2. 测试隔离

- 每个测试使用独立的临时文件和目录
- 不依赖其他测试的执行结果
- 清理测试产生的副作用

### 3. 测试数据管理

- 使用 `test_data` 模块中的预定义数据
- 为特殊场景创建专门的测试数据
- 避免硬编码测试数据

### 4. 错误处理测试

- 测试正常情况和异常情况
- 验证错误消息的准确性
- 确保系统能优雅地处理错误

### 5. 性能测试

- 为关键功能添加性能测试
- 设置合理的性能基准
- 监控测试执行时间

## 添加新测试

### 1. 确定测试类型

根据测试目的选择合适的测试类型：

- 测试单个函数 → 单元测试
- 测试模块交互 → 集成测试
- 测试用户场景 → 端到端测试

### 2. 选择测试文件

将测试添加到相应的文件中：

- 编号相关 → `numbering_tests.rs` 或 `numbering_integration_tests.rs`
- 图片相关 → `image_localization_*_tests.rs`
- 解析器相关 → `parser_tests.rs`
- 验证相关 → `validation_tests.rs`
- 工作流相关 → `workflow_tests.rs`

### 3. 使用公共工具

利用 `common` 模块中的工具简化测试编写：

- 使用 `TestFileManager` 管理文件
- 使用配置构建器创建配置
- 使用断言辅助函数验证结果

### 4. 遵循命名规范

按照既定的命名规范命名测试函数。

## 故障排除

### 常见问题

1. **测试文件冲突**
   - 确保每个测试使用独立的临时目录
   - 检查是否有测试遗留的文件

2. **Mock 服务器问题**
   - 确保启用了 `mock` feature
   - 检查网络端口是否被占用

3. **性能测试失败**
   - 调整性能基准以适应不同的硬件环境
   - 考虑系统负载对测试的影响

4. **并发测试问题**
   - 减少并行测试线程数
   - 确保测试间没有共享资源冲突

### 调试技巧

1. **使用 `--nocapture` 查看输出**
2. **添加 `println!` 调试信息**
3. **使用 `cargo test -- --exact test_name` 运行单个测试**
4. **检查临时文件内容以验证中间状态**

## 持续集成

测试应该在以下情况下自动运行：

- 代码提交时
- 拉取请求时
- 定期构建时

确保所有测试在 CI 环境中都能稳定通过。
