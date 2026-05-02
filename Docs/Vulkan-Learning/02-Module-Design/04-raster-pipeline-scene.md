# 模块四：光栅管线、着色器、材质与场景绘制

导航：
- [模块设计索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/README.md)
- [本模块实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/04-raster-pipeline-scene.md)
- [本模块可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/04-raster-pipeline-scene.md)
- [本模块开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/04-raster-pipeline-scene.md)

## 模块职责

这个模块负责把场景数据通过传统光栅化绘制出来：

- shader module 与 shader 编译流程
- pipeline layout 与 graphics pipeline
- vertex / index buffer
- depth buffer
- uniform / storage buffer
- texture / sampler
- mesh / material / draw item
- camera 与 per-frame GPU 参数
- forward、deferred 或 PBR 基础 pass

## 关键设计问题

### 1. Render pass 还是 dynamic rendering

学习 Vulkan 基础时可以理解传统 render pass；实现主线建议尽早评估 dynamic rendering，减少 framebuffer/render pass 组合爆炸。无论选择哪条路，都必须明确 attachment load/store 和 layout。

### 2. Shader 语言怎么选

第一版可用 GLSL 编译 SPIR-V，保持入门成本低。后续可以评估 HLSL + DXC 或 Slang，但 shader 输入输出接口必须由文档和反射工具约束。

### 3. 场景数据如何进 GPU

不要让每个 mesh 自己更新 descriptor。建议建立：

- 全局 camera / lighting buffer
- material buffer 或 material descriptor
- mesh buffer
- draw item 列表

### 4. PBR 何时加入

先做 unlit / Lambert / Blinn-Phong，确认 buffer、texture、depth、camera 和多物体绘制稳定后，再进入 metallic-roughness PBR。

## 建议数据结构

- `ShaderModule`
  - stage
  - module handle
  - entry point
- `GraphicsPipelineDesc`
  - shader stages
  - vertex layout
  - raster state
  - depth state
  - color targets
  - pipeline layout
- `MeshGpu`
  - vertex buffer
  - index buffer
  - index count
- `MaterialGpu`
  - base color
  - texture handles
  - roughness / metallic
- `DrawItem`
  - mesh
  - material
  - transform index
- `SceneGpuData`
  - camera buffer
  - transform buffer
  - light buffer

## 模块接口建议

- `load_shader(path, stage) -> ShaderModuleHandle`
- `create_graphics_pipeline(desc) -> PipelineHandle`
- `upload_mesh(mesh) -> MeshHandle`
- `upload_texture(image) -> TextureHandle`
- `create_material(desc) -> MaterialHandle`
- `draw_scene(cmd, scene, camera, target)`

## 关键不变量

- shader 中 descriptor set / binding 必须与 pipeline layout 一致。
- vertex buffer layout 必须与 shader 输入一致。
- depth image layout 必须在 depth pass 中正确转换。
- pipeline 不能引用已销毁的 shader module、layout 或 render target 语义。
- 每个 draw item 必须能追溯到 mesh、material 和 transform。

## 与其他模块的耦合点

- 依赖模块二创建 buffer、image、descriptor。
- 依赖模块三录制 draw、barrier 和 pass begin/end。
- 向模块五提供 mesh、material 和场景数据，用于 BLAS/TLAS 和 hit shader。
- 向模块六提供 pass 级别的输入输出声明。

## 参考资料

- Khronos Tutorial：graphics pipeline、vertex buffer、texture mapping、depth buffering。
- Vulkan Samples：dynamic rendering、pipeline cache、descriptor indexing。
- glTF 2.0 PBR 材质模型可作为后续资产格式参考。

## 设计取舍

- 第一版 draw path 可以简单，但数据结构要能扩展到多 mesh / 多 material。
- 不要在 shader 里硬编码太多全局状态，尽早建立清晰 descriptor 约定。
- PBR 不是优先级最高；稳定的资源与同步模型更重要。
