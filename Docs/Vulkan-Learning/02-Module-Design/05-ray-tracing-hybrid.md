# 模块五：光线追踪、加速结构、SBT 与混合渲染

导航：
- [模块设计索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/README.md)
- [本模块实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/05-ray-tracing-hybrid.md)
- [本模块可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/05-ray-tracing-hybrid.md)
- [本模块开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/05-ray-tracing-hybrid.md)

## 模块职责

这个模块负责 Vulkan 硬件光追能力：

- ray tracing feature / extension 查询
- buffer device address
- BLAS / TLAS 构建与更新
- scratch buffer 管理
- ray tracing shader module
- ray tracing pipeline 与 shader group
- shader binding table
- `vkCmdTraceRaysKHR`
- ray query 混合光栅技术
- 光追输出与光栅结果合成

## 关键设计问题

### 1. 先 ray tracing pipeline 还是 ray query

建议先做 ray tracing pipeline，因为它迫使你理解 SBT、shader group、raygen/miss/hit 的完整模型。之后再用 ray query 做光栅 shader 中的阴影或 AO。

### 2. BLAS/TLAS 生命周期如何管理

BLAS 通常跟 mesh 几何绑定，TLAS 跟场景 instance 绑定。mesh 静态时 BLAS 可长期存在；transform 或 instance 变化时更新 TLAS。

### 3. SBT 为什么是独立资源

SBT 是 ray tracing pipeline 的调度表，里面放 shader group handle 和可选 record data。它有严格的对齐、stride 和 device address 要求，必须作为专门资源管理。

### 4. 光追结果如何进入光栅链路

第一版让光追 pass 写入 storage image，再由 fullscreen composite pass 采样或读取它。这样可以清楚观察 pass 边界和 layout transition。

## 建议数据结构

- `RayTracingCapabilities`
  - supported extensions
  - shader group handle size
  - shader group base alignment
  - max recursion depth
- `AccelerationStructure`
  - handle
  - backing buffer
  - device address
  - size
  - kind
- `BlasBuildInput`
  - vertex buffer address
  - index buffer address
  - geometry flags
- `TlasInstance`
  - transform
  - blas address
  - instance custom index
  - mask
  - hit group offset
- `ShaderBindingTable`
  - raygen region
  - miss region
  - hit region
  - callable region
  - backing buffer
- `RayTracingPass`
  - pipeline
  - pipeline layout
  - descriptor sets
  - output image

## 模块接口建议

- `query_ray_tracing_capabilities(context) -> RayTracingCapabilities`
- `build_blas(meshes) -> BlasHandle`
- `build_tlas(instances) -> TlasHandle`
- `create_ray_tracing_pipeline(desc) -> RayPipelineHandle`
- `create_shader_binding_table(pipeline) -> ShaderBindingTable`
- `trace_rays(cmd, pass, extent)`
- `ray_query_shadow_pass(cmd, scene, gbuffer)`

## 关键不变量

- acceleration structure backing buffer 必须有正确 usage 和 device address。
- scratch buffer 生命周期必须覆盖 build 命令执行。
- TLAS instance 引用的 BLAS 必须在 trace 时仍然有效。
- SBT region 的 address、stride、size 必须满足设备对齐要求。
- 光追输出 image 必须在 trace 前是 general 或正确 storage layout，合成前要转换为可读 layout。

## 与其他模块的耦合点

- 依赖模块二提供 device address buffer、storage image、descriptor。
- 依赖模块三提供 acceleration build 和 trace 的同步。
- 复用模块四的 mesh、material、transform 和 camera 数据。
- 通过模块六接入 render graph，实现光栅和光追 pass 的资源依赖。

## 参考资料

- Vulkan Ray Tracing Guide：五个 KHR 扩展、BLAS/TLAS、SBT、同步。
- Vulkan Samples：ray tracing basic。
- Khronos Ray Tracing Best Practices：payload、device-local memory、ray query 使用建议。

## 设计取舍

- 第一版只做三角形或简单 mesh 的单 bounce 光追。
- 第一版可以不做 compaction 和 update 优化。
- 不要在光追阶段补救前面资源系统的缺陷；资源和同步不稳时先回模块二、三修地基。
