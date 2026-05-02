# 模块五开发计划：光线追踪、加速结构、SBT 与混合渲染

导航：
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/05-ray-tracing-hybrid.md)
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/05-ray-tracing-hybrid.md)

## 推荐开发顺序

| 顺序 | 子任务 | 建议 Issue 标题 | 前置条件 | 测试重点 | 完成定义 |
| --- | --- | --- | --- | --- | --- |
| 1 | M5-S1 | `feat: query ray tracing feature support` | M1-S7 | extension/feature | 支持矩阵清晰 |
| 2 | M5-S2 | `feat: query ray tracing device properties` | M5-S1 | SBT 对齐 | property 可用 |
| 3 | M5-S3 | `feat: allocate acceleration structure buffers` | M2-S14 | usage/address | AS buffer 正确 |
| 4 | M5-S4 | `feat: build triangle blas` | M5-S3, M3-S12 | build command | BLAS 成功 |
| 5 | M5-S5 | `feat: build mesh blas inputs` | M4-S7 | vertex/index address | mesh 可光追 |
| 6 | M5-S6 | `feat: upload tlas instances` | M5-S4 | instance layout | instance buffer 正确 |
| 7 | M5-S7 | `feat: build top level acceleration structure` | M5-S6 | TLAS 引用 | TLAS 成功 |
| 8 | M5-S8 | `build: compile ray tracing shaders` | M5-S2 | shader stages | RT shader 可加载 |
| 9 | M5-S9 | `feat: create ray tracing pipeline` | M5-S8, M2-S13 | shader groups | pipeline 成功 |
| 10 | M5-S10 | `feat: fetch ray tracing shader group handles` | M5-S9 | handle count | group handles 正确 |
| 11 | M5-S11 | `feat: create shader binding table` | M5-S10 | address/stride | SBT 合法 |
| 12 | M5-S12 | `feat: create ray tracing output image` | M2-S8 | storage layout | 输出图可写 |
| 13 | M5-S13 | `demo: trace rays to output image` | M5-S11, M5-S12 | trace 参数 | 光追结果可见 |
| 14 | M5-S14 | `demo: composite ray tracing with raster` | M4-S14, M5-S13 | pass 依赖 | 混合渲染完成 |

## 实施建议

- 不支持硬件光追的机器要能跳过相关 demo，而不是让整个项目不可运行。
- 每个 acceleration structure build 都要记录 scratch buffer 和 backing buffer 生命周期。
- SBT 是最容易出错的区域，所有 alignment 都要从 device properties 读，不要硬编码。

## 每个子任务的固定动作

1. 记录需要开启的 extension、feature 和 property。
2. 说明 AS / SBT buffer 的 usage、address 和销毁顺序。
3. 运行 `cargo fmt --all -- --check`。
4. 运行 `cargo clippy --all-targets --all-features -- -D warnings`。
5. 用 validation layer 和 RenderDoc 验证 trace pass。
