# 模块四开发计划：光栅管线、着色器、材质与场景绘制

导航：
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/04-raster-pipeline-scene.md)
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/04-raster-pipeline-scene.md)

## 推荐开发顺序

| 顺序 | 子任务 | 建议 Issue 标题 | 前置条件 | 测试重点 | 完成定义 |
| --- | --- | --- | --- | --- | --- |
| 1 | M4-S1 | `build: compile raster shaders to spirv` | M1-S13 | 编译错误日志 | SPIR-V 可生成 |
| 2 | M4-S2 | `feat: load vulkan shader modules` | M4-S1 | module 创建/销毁 | shader 可加载 |
| 3 | M4-S3 | `docs: define shader descriptor bindings` | M2-S11 | set/binding 一致 | 绑定表清晰 |
| 4 | M4-S4 | `feat: create raster pipeline layouts` | M4-S3 | layout 生命周期 | layout 可复用 |
| 5 | M4-S5 | `feat: create graphics pipelines` | M4-S2, M4-S4 | pipeline state | pipeline 可创建 |
| 6 | M4-S6 | `demo: draw first raster triangle` | M4-S5, M3-S9 | draw/viewport | 三角形可见 |
| 7 | M4-S7 | `feat: upload mesh vertex data` | M2-S9 | vertex layout | mesh 可上传 |
| 8 | M4-S8 | `feat: draw indexed meshes` | M4-S7 | index buffer | indexed draw 正确 |
| 9 | M4-S9 | `feat: build raster draw item list` | M4-S8 | 多物体 | draw item 可遍历 |
| 10 | M4-S10 | `feat: add depth testing` | M2-S8, M3-S10 | depth layout | 遮挡正确 |
| 11 | M4-S11 | `feat: sample uploaded textures` | M2-S10, M2-S13 | sampler/descriptor | 贴图可见 |
| 12 | M4-S12 | `feat: bind material parameters` | M4-S11 | 材质变化 | material 生效 |
| 13 | M4-S13 | `feat: update scene gpu buffers` | M2-S13 | frames in flight | camera/transform 正确 |
| 14 | M4-S14 | `demo: render basic pbr scene` | M4-S13 | 多 pass/参数 | 光栅主线完整 |

## 实施建议

- first triangle 只证明 pipeline 和帧循环；不要把它当最终架构。
- shader binding 表要和 Rust 结构一起维护。
- 进入 PBR 前必须确保 depth、texture、camera、material 都已独立验证。

## 每个子任务的固定动作

1. 更新 shader binding 文档。
2. 用验证层检查 descriptor、pipeline 和 render target。
3. 运行 `cargo fmt --all -- --check`。
4. 运行 `cargo clippy --all-targets --all-features -- -D warnings`。
5. 对画面任务，记录预期截图或 RenderDoc 捕获点。
