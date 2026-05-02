# 模块二开发计划：GPU 内存、资源、上传路径与描述符

导航：
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/02-memory-resource-descriptor.md)
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/02-memory-resource-descriptor.md)

## 推荐开发顺序

| 顺序 | 子任务 | 建议 Issue 标题 | 前置条件 | 测试重点 | 完成定义 |
| --- | --- | --- | --- | --- | --- |
| 1 | M2-S1 | `feat: inspect gpu memory properties` | M1-S8 | heap/type 日志 | 内存信息可读 |
| 2 | M2-S2 | `feat: select vulkan memory types` | M2-S1 | property 匹配 | type 选择可测 |
| 3 | M2-S3 | `feat: define gpu allocation ownership` | M2-S2 | 销毁顺序 | allocation RAII 稳定 |
| 4 | M2-S4 | `feat: create bound gpu buffers` | M2-S3 | requirements/bind | buffer 可用 |
| 5 | M2-S5 | `feat: write host visible buffers` | M2-S4 | map/flush | CPU 写入正确 |
| 6 | M2-S6 | `feat: upload device local mesh buffers` | M2-S5, M3-S12 | copy 验证 | 顶点/索引可上传 |
| 7 | M2-S7 | `feat: create gpu images and samplers` | M2-S3 | image view/sampler | texture 资源完整 |
| 8 | M2-S8 | `feat: create depth and storage images` | M2-S7 | usage/layout | 深度和输出图可用 |
| 9 | M2-S9 | `feat: stage buffer uploads` | M2-S6 | copy buffer | 上传路径闭环 |
| 10 | M2-S10 | `feat: stage texture uploads` | M2-S7, M3-S10 | copy image/layout | 贴图上传闭环 |
| 11 | M2-S11 | `feat: cache descriptor set layouts` | M2-S4 | layout key | layout 可复用 |
| 12 | M2-S12 | `feat: allocate descriptor sets` | M2-S11 | pool reset | set 可分配 |
| 13 | M2-S13 | `feat: write descriptor resources` | M2-S12 | buffer/image binding | shader 可访问资源 |
| 14 | M2-S14 | `feat: expose buffer device addresses` | M2-S4 | usage/feature | 光追前置完成 |

## 实施建议

- 手写 allocator 阶段只追求正确和可解释。
- 每种资源类型都写清楚 usage flags，避免“能跑但不知道为什么”。
- descriptor helper 要保持透明，能看出 set、binding、array element 和资源类型。

## 每个子任务的固定动作

1. 补充资源创建失败时的错误信息。
2. 记录 memory property、usage flags 和生命周期。
3. 运行 `cargo fmt --all -- --check`。
4. 运行 `cargo clippy --all-targets --all-features -- -D warnings`。
5. 对上传任务，用 RenderDoc 或验证层确认资源状态正确。
