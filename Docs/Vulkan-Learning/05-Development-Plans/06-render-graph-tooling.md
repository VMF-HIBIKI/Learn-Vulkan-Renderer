# 模块六开发计划：渲染图、调试工具、性能分析与工程化

导航：
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/06-render-graph-tooling.md)
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/06-render-graph-tooling.md)

## 推荐开发顺序

| 顺序 | 子任务 | 建议 Issue 标题 | 前置条件 | 测试重点 | 完成定义 |
| --- | --- | --- | --- | --- | --- |
| 1 | M6-S1 | `feat: name vulkan objects for debugging` | M1-S4 | debug utils 可选 | 对象名称可见 |
| 2 | M6-S2 | `feat: add gpu debug label scopes` | M3-S3 | label 成对 | pass marker 可见 |
| 3 | M6-S3 | `feat: define render graph pass nodes` | M4-S6 | pass registry | pass 可登记 |
| 4 | M6-S4 | `feat: declare render graph resource access` | M6-S3 | read/write 声明 | 依赖可见 |
| 5 | M6-S5 | `feat: compile render graph schedule` | M6-S4 | 顺序稳定 | graph 可执行 |
| 6 | M6-S6 | `feat: track render graph resource states` | M6-S5 | 状态历史 | resource state 可查 |
| 7 | M6-S7 | `feat: emit render graph barriers` | M6-S6 | barrier 正确 | 同步可自动化 |
| 8 | M6-S8 | `build: rebuild shaders from project scripts` | M4-S1 | 增量编译 | shader 工具可用 |
| 9 | M6-S9 | `feat: persist vulkan pipeline cache` | M4-S5 | cache 文件 | cache 可复用 |
| 10 | M6-S10 | `feat: create gpu timestamp queries` | M3-S6 | query lifecycle | timestamp 可写 |
| 11 | M6-S11 | `feat: report gpu pass timings` | M6-S10 | 等待/读取 | pass 时间可读 |
| 12 | M6-S12 | `demo: organize renderer regression demos` | M4-S14 | 启动约定 | demo 可回归 |
| 13 | M6-S13 | `bench: add renderer benchmark scene` | M6-S11 | 基线记录 | 性能可比较 |

## 实施建议

- Debug name 和 label 越早接入越好，不必等 render graph。
- Render graph 初版只解决顺序和 barrier，不急着做资源别名。
- Benchmark 要和具体场景绑定，否则数字没有解释力。

## 每个子任务的固定动作

1. 确认新增工具不会在 release 配置中造成不可接受开销。
2. 对调试功能提供“不可用时降级”的路径。
3. 运行 `cargo fmt --all -- --check`。
4. 运行 `cargo clippy --all-targets --all-features -- -D warnings`。
5. 更新 demo 或 benchmark 的预期输出说明。
