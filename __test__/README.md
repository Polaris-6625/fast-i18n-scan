# 测试目录

本目录包含项目的所有测试文件。

## 目录结构

- `unit/` - 单元测试（原tests目录）
- `integration/` - 集成测试
- `fixtures/` - 测试数据和夹具

## 运行测试

使用统一的测试脚本：

```bash
./scripts/test.sh
```

或者单独运行：

```bash
# 单元测试
cargo test

# 集成测试  
cargo test --test '*'