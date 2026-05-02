# 学习与实现路线图

导航：
- [第一层索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/01-Macro-Overview/README.md)
- [第三层实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/README.md)

## 总体阶段

### 阶段 0：术语、工具和硬件边界先统一

产出：

- 明确 Vulkan SDK、验证层、RenderDoc、shader 编译工具链。
- 明确目标平台、GPU、driver、Vulkan 版本和光追扩展支持情况。
- 明确 Rust 绑定、窗口库、shader 语言和资源加载策略。

### 阶段 1：最小可运行 Vulkan 闭环

目标是让 Vulkan 程序先“活起来”：

- 创建 window / surface。
- 创建 instance / debug messenger。
- 选择 physical device 和 queue family。
- 创建 logical device、graphics queue、present queue。
- 创建 swapchain 并清屏呈现。

### 阶段 2：资源与同步基础

目标是摆脱硬编码 demo，建立真实渲染器地基：

- buffer / image 创建与内存绑定。
- staging upload。
- descriptor set / descriptor pool。
- command buffer。
- fence / semaphore / image barrier。

### 阶段 3：光栅化渲染器

目标是从三角形推进到可扩展光栅管线：

- shader 编译与 pipeline layout。
- vertex / index buffer。
- depth buffer。
- texture / sampler。
- camera uniform。
- mesh / material 批量绘制。

### 阶段 4：渲染架构工程化

目标是让渲染代码不再堆在 main loop：

- frame context。
- resource registry。
- render pass abstraction。
- render graph。
- pipeline cache。
- debug name 与 GPU marker。

### 阶段 5：PBR 与多 pass 光栅

目标是接近现代实时渲染主线：

- GBuffer 或 forward+ 基础。
- PBR 材质参数。
- IBL 或环境光。
- shadow map。
- post-process。
- HDR / tone mapping。

### 阶段 6：硬件光线追踪

目标是建立 Vulkan RT 的最小闭环：

- 检查 ray tracing feature 和 extension。
- 创建 BLAS / TLAS。
- 创建 ray tracing pipeline。
- 创建 shader binding table。
- 输出 ray traced shadow / reflection / AO。

### 阶段 7：混合渲染器与性能迭代

目标是让光栅和光追协同工作：

- 光栅 GBuffer 提供光追输入。
- 光追 pass 输出中间图像。
- 合成 pass 混合最终画面。
- 使用 timestamp / RenderDoc / vendor profiler 分析瓶颈。
- 对资源分配、descriptor、barrier 和 pass 顺序做迭代。

## 每一阶段的学习原则

- 先实现最小闭环，再扩展能力。
- 每增加一个 Vulkan 对象，都要记录创建、使用、销毁和同步责任。
- 每个阶段必须有可运行 demo 或可观测验证结果。
- 任何“暂时能跑”的同步代码，都要问清楚是否会阻塞后面的 render graph 和 ray tracing。

## 推荐实践节奏

1. 先从第三层选一个大任务。
2. 到第四层确认它被拆成哪些可执行动作。
3. 到第五层按子任务开发计划推进。
4. 写完后回到第二层，确认设计是否需要修正。

## 下一步

- 进入 [第二层模块设计索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/README.md)。
