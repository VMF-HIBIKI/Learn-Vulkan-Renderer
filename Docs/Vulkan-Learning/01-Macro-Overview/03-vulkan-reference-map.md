# Vulkan 参考映射

导航：
- [第一层索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/01-Macro-Overview/README.md)
- [第二层模块设计](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/README.md)

## 官方资料优先级

学习 Vulkan 时，资料优先级建议如下：

1. Vulkan Specification：最终事实来源。
2. Vulkan Guide：解释概念、扩展和最佳实践。
3. Khronos Vulkan Tutorial：建立第一条完整渲染路径。
4. Vulkan Samples：查看真实扩展和功能样例。
5. Ash 文档与源码：理解 Rust 绑定如何暴露 Vulkan。

## 模块到资料的映射

| 模块 | 主要参考 | 使用方式 |
| --- | --- | --- |
| M1 平台与交换链 | Khronos Tutorial、Vulkan Guide、Ash examples | 建立 instance、surface、device、swapchain 的第一闭环 |
| M2 内存与资源 | Vulkan Guide memory、descriptor、buffer/image 章节 | 明确分配、绑定、上传和 descriptor 生命周期 |
| M3 命令与同步 | Vulkan synchronization 文档、validation layer 输出 | 建立 command buffer、barrier、semaphore、fence 的可验证模型 |
| M4 光栅管线 | Tutorial graphics pipeline、Samples render pass / dynamic rendering | 从三角形推进到 mesh、depth、material 和 PBR |
| M5 光线追踪 | Vulkan Ray Tracing Guide、ray tracing basic sample | 建立 BLAS、TLAS、SBT、ray tracing pipeline 与 ray query |
| M6 工程化 | RenderDoc、debug utils、pipeline cache、timestamp query | 把 demo 变成可调试、可维护、可测量的渲染器 |

## Rust 绑定基线

本项目默认以 `ash` 作为底层 Vulkan 绑定学习对象。原因：

- 它接近原始 Vulkan API，适合学习底层机制。
- 它不会替你隐藏同步、内存和 descriptor 的复杂度。
- 它提供 Rust 类型封装，但仍明确要求调用方负责 Vulkan 安全不变量。

如果后续需要更高级的实验，可以单独比较 `vulkano`、`wgpu` 或自建抽象层，但本项目主线不依赖它们。

## 光追扩展基线

硬件光追主线围绕以下 Vulkan KHR 扩展理解：

- `VK_KHR_acceleration_structure`
- `VK_KHR_ray_tracing_pipeline`
- `VK_KHR_ray_query`
- `VK_KHR_pipeline_library`
- `VK_KHR_deferred_host_operations`

第一版光追目标不要追求复杂效果，先验证：

- 支持目标 GPU 能力查询。
- 创建 BLAS / TLAS。
- 创建 ray generation、miss、closest hit shader。
- 构建 shader binding table。
- 执行 `vkCmdTraceRaysKHR` 并写入 storage image。

## 工具基线

- Vulkan SDK：验证层、shader 工具、运行时 loader。
- RenderDoc：帧捕获、资源状态、pipeline 检查。
- GPU vendor 工具：用于后期性能分析，不作为第一阶段前置。
- `cargo fmt` / `cargo clippy` / `cargo test`：保证 Rust 工程质量。

## 阅读原则

- 先用 tutorial 建立最小闭环，再回到 specification 查边界条件。
- 遇到同步和内存问题，优先相信验证层和 RenderDoc。
- 每学一个扩展，都要回答它新增了哪些 handle、哪些命令、哪些同步需求、哪些 shader 能力。
