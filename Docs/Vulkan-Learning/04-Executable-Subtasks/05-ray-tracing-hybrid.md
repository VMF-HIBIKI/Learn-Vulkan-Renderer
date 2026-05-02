# 模块五可执行子任务：光线追踪、加速结构、SBT 与混合渲染

导航：
- [第三层实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/05-ray-tracing-hybrid.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/05-ray-tracing-hybrid.md)

## 子任务表

| 子任务 ID | 对应大任务 | 子任务内容 | 完成标志 |
| --- | --- | --- | --- |
| M5-S1 | M5-T1 | 查询 RT extension 和 feature chain | 支持矩阵可打印 |
| M5-S2 | M5-T1 | 查询 ray tracing pipeline properties | SBT 对齐信息可用 |
| M5-S3 | M5-T2 | 创建 AS backing buffer 和 scratch buffer | buffer usage 正确 |
| M5-S4 | M5-T2 | 构建三角形 BLAS | BLAS build 成功 |
| M5-S5 | M5-T2 | 从 mesh 数据构建 BLAS input | mesh 可进入光追 |
| M5-S6 | M5-T3 | 创建 TLAS instance buffer | instance 数据上传成功 |
| M5-S7 | M5-T3 | 构建 TLAS | TLAS build 成功 |
| M5-S8 | M5-T4 | 编译 raygen/miss/closest-hit shader | RT shader 可加载 |
| M5-S9 | M5-T4 | 创建 ray tracing pipeline | pipeline 创建成功 |
| M5-S10 | M5-T5 | 查询 shader group handles | handle 数据正确 |
| M5-S11 | M5-T5 | 创建 SBT buffer 和 region | SBT 对齐满足设备要求 |
| M5-S12 | M5-T6 | 创建 storage image 输出 | trace 目标可写 |
| M5-S13 | M5-T6 | 执行 `trace_rays` 并显示结果 | 画面出现光追输出 |
| M5-S14 | M5-T6 | 与光栅 pass 合成 | 混合渲染闭环 |

## 执行提示

- M5-S1 到 M5-S2 先确认硬件和 driver 能力，避免盲写。
- M5-S3 到 M5-S7 只解决 acceleration structure。
- M5-S8 到 M5-S14 才进入 shader、SBT 和画面输出。
