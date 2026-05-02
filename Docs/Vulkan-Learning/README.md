# Vulkan Learning

这套文档的目标不是“照着教程画一个三角形”，而是把“使用 Rust 从零学习 Vulkan，并逐步实现一个支持光栅化与光线追踪的渲染器”拆成可学习、可执行、可验证的长期工程。

## 导航索引

- 第一层：宏观理解与术语地图
  - [总览索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/01-Macro-Overview/README.md)
- 第二层：模块设计
  - [模块设计索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/README.md)
- 第三层：模块实现大任务
  - [实现大任务索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/README.md)
- 第四层：可执行子任务
  - [可执行子任务索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/README.md)
- 第五层：子任务开发计划
  - [开发计划索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/README.md)

## 六个核心模块

- 模块一：Vulkan 平台、实例、物理设备与交换链
- 模块二：GPU 内存、资源、上传路径与描述符
- 模块三：命令录制、同步模型与帧循环
- 模块四：光栅管线、着色器、材质与场景绘制
- 模块五：光线追踪、加速结构、SBT 与混合渲染
- 模块六：渲染图、调试工具、性能分析与工程化

## 如何使用这套文档

1. 先读第一层，建立 Vulkan 的显式 API 心智模型。
2. 再读第二层，明确渲染器会由哪些模块组成，以及每个模块拥有哪些资源。
3. 做实现时只看第三层和第四层，把大问题压成单次可完成的任务。
4. 开工前对照第五层，先写测试、验证层检查方式和验收标准，再开始编码。
5. 每次只推进一个子任务，保持“一次只吃透一个 GPU 机制”的节奏。

## 参考基线

- Khronos Vulkan Guide：<https://docs.vulkan.org/guide/latest/index.html>
- Khronos Vulkan Tutorial：<https://docs.vulkan.org/tutorial/latest/00_Introduction.html>
- Vulkan Ray Tracing Guide：<https://github.khronos.org/Vulkan-Site/guide/latest/extensions/ray_tracing.html>
- Khronos Vulkan Samples：<https://github.khronos.org/Vulkan-Site/samples/latest/>
- Ash Rust Vulkan 绑定：<https://github.com/ash-rs/ash>

## 目标产出

- 形成一套完整的 Vulkan 渲染器学习图谱，而不是零散笔记。
- 为后续 `src/` 中的 Vulkan 封装、渲染器核心和 demo 提供稳定设计约束。
- 让每个 Issue、每个 Commit 都能明确对应到一个具体学习目标与渲染能力。
- 最终做出一个具备基础 PBR 光栅、硬件光追阴影/反射、调试视图和性能分析能力的学习型渲染器。
