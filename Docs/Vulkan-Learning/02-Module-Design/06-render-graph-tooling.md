# 模块六：渲染图、调试工具、性能分析与工程化

导航：
- [模块设计索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/README.md)
- [本模块实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/06-render-graph-tooling.md)
- [本模块可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/06-render-graph-tooling.md)
- [本模块开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/06-render-graph-tooling.md)

## 模块职责

这个模块负责把前五个模块从“能跑的 demo”提升为“可维护的渲染器工程”：

- render graph
- pass 输入输出声明
- 资源状态推导
- transient resource
- debug name / debug label
- validation layer 集成策略
- shader hot reload
- pipeline cache
- GPU timestamp / statistics
- RenderDoc 捕获约定
- 性能基准和回归检查

## 关键设计问题

### 1. Render graph 何时引入

不要第一天就写复杂 render graph。等至少有 swapchain clear、depth、mesh draw、post process 或 ray tracing output 后，再抽象 pass 依赖。

### 2. Render graph 管资源还是只管依赖

第一版可以只管 pass 顺序、资源读写声明和 barrier 生成。后续再加入 transient resource 创建、别名和生命周期优化。

### 3. Debug 工具如何成为默认流程

每个 Vulkan handle 创建后尽量设置 debug name。每个 pass 录制时插入 debug label。这样 RenderDoc 中的帧才可读。

### 4. 性能优化怎么避免瞎猜

先加 timestamp query 和基础统计，再谈优化。重点指标包括：

- CPU frame time
- GPU pass time
- draw count
- descriptor update count
- buffer/image allocation count
- BLAS/TLAS build time

## 建议数据结构

- `RenderGraph`
  - pass list
  - resource declarations
  - dependency edges
  - compiled schedule
- `RenderPassNode`
  - name
  - reads
  - writes
  - execute callback
- `GraphResource`
  - logical handle
  - physical resource
  - usage history
- `DebugContext`
  - debug utils loader
  - naming helpers
  - label helpers
- `GpuProfiler`
  - timestamp query pool
  - frame results
  - pass timing records

## 模块接口建议

- `graph.add_pass(name, reads, writes, execute)`
- `graph.compile(resource_registry) -> CompiledGraph`
- `compiled_graph.execute(frame, cmd)`
- `set_debug_name(handle, name)`
- `begin_debug_label(cmd, name)`
- `end_debug_label(cmd)`
- `profiler.begin_pass(cmd, name)`
- `profiler.end_pass(cmd, name)`

## 关键不变量

- 一个 pass 的读写资源必须在执行前声明。
- graph 编译出的 barrier 必须能解释每个资源状态变化。
- debug label 必须成对 begin/end。
- timestamp 结果读取不能早于 GPU 完成。
- 性能结论必须基于测量数据，而不是视觉感觉。

## 与其他模块的耦合点

- 汇总模块二的资源注册信息。
- 使用模块三的 barrier 和 command recording 接口。
- 组织模块四和模块五的实际渲染 pass。
- 反向推动模块一到五暴露更清晰的生命周期和调试信息。

## 参考资料

- Vulkan Guide：debug utils、queries、synchronization。
- RenderDoc 文档和实际帧捕获。
- Vulkan Samples：timestamp、pipeline cache、debug marker 相关样例。

## 设计取舍

- 第一版 render graph 不追求自动化到极致，重点是让依赖可见。
- Debug 和 profiling 不是后期装饰，而是 Vulkan 项目的生存工具。
- 工程化模块不应该反过来污染底层 Vulkan wrapper；它应通过清晰接口组合已有能力。
