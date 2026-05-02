# 模块四可执行子任务：光栅管线、着色器、材质与场景绘制

导航：
- [第三层实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/04-raster-pipeline-scene.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/04-raster-pipeline-scene.md)

## 子任务表

| 子任务 ID | 对应大任务 | 子任务内容 | 完成标志 |
| --- | --- | --- | --- |
| M4-S1 | M4-T1 | 建立 shader 目录和编译命令 | SPIR-V 可生成 |
| M4-S2 | M4-T1 | 加载 shader module | shader module 可创建 |
| M4-S3 | M4-T1 | 定义 descriptor set layout 约定 | Rust/shader binding 一致 |
| M4-S4 | M4-T2 | 创建 pipeline layout | pipeline layout 可复用 |
| M4-S5 | M4-T2 | 创建 graphics pipeline | pipeline 创建成功 |
| M4-S6 | M4-T2 | 绘制 first triangle | 屏幕可见三角形 |
| M4-S7 | M4-T3 | 定义 vertex layout 和 mesh upload | mesh 数据进入 GPU |
| M4-S8 | M4-T3 | 实现 indexed draw | index buffer 正常 |
| M4-S9 | M4-T3 | 定义 draw item 列表 | 多物体可绘制 |
| M4-S10 | M4-T4 | 创建 depth buffer 并开启深度测试 | 遮挡正确 |
| M4-S11 | M4-T4 | 上传 texture 并创建 sampler | 贴图可采样 |
| M4-S12 | M4-T4 | 定义 material 数据和 descriptor | 材质参数生效 |
| M4-S13 | M4-T5 | 上传 camera / transform / light buffer | 场景参数每帧更新 |
| M4-S14 | M4-T6 | 实现 PBR 初版或多 pass demo | 光栅主线可扩展 |

## 执行提示

- M4-S1 到 M4-S6 只追求第一条 graphics pipeline 跑通。
- M4-S7 到 M4-S13 把 demo 推进到真实场景数据。
- M4-S14 开始进入算法，不要跳过前面的资源和同步验收。
