# 模块四实现大任务：光栅管线、着色器、材质与场景绘制

导航：
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/04-raster-pipeline-scene.md)
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/04-raster-pipeline-scene.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/04-raster-pipeline-scene.md)

## 大任务总览

| 任务 ID | 名称 | 目标 |
| --- | --- | --- |
| M4-T1 | Shader 与 Pipeline Layout | 建立 shader 编译、加载和 descriptor 接口 |
| M4-T2 | Graphics Pipeline | 创建可绘制三角形的图形管线 |
| M4-T3 | Mesh 与 Draw Item | 支持 vertex/index buffer 和多物体绘制 |
| M4-T4 | Depth、Texture 与 Material | 支持深度测试、贴图采样和材质参数 |
| M4-T5 | Camera 与 Scene GPU Data | 建立 camera、transform、light 的 GPU 数据路径 |
| M4-T6 | PBR 与多 Pass 基础 | 推进到可扩展的真实光栅渲染 |

## M4-T1 Shader 与 Pipeline Layout

产出：

- shader 目录约定
- SPIR-V 编译脚本或 build 流程
- pipeline layout 创建
- descriptor set layout 绑定表

验收标准：

- shader 编译失败能给出可读日志
- shader binding 与 Rust descriptor 定义一致
- pipeline layout 可复用

## M4-T2 Graphics Pipeline

产出：

- vertex shader / fragment shader
- graphics pipeline desc
- render target / depth state / raster state
- first triangle demo

验收标准：

- 能绘制非全屏三角形
- 验证层无 pipeline / render target 错误
- resize 后 pipeline 或 target 状态正确

## M4-T3 Mesh 与 Draw Item

产出：

- mesh upload
- vertex layout
- index draw
- draw item 列表

验收标准：

- 能绘制多个 mesh
- index buffer 正确工作
- draw item 能关联 mesh、material、transform

## M4-T4 Depth、Texture 与 Material

产出：

- depth image
- texture upload
- sampler
- material descriptor

验收标准：

- 深度测试正确遮挡
- texture 采样正常
- material 参数能影响 fragment 输出

## M4-T5 Camera 与 Scene GPU Data

产出：

- camera uniform / storage buffer
- transform buffer
- light buffer
- 每帧更新路径

验收标准：

- 相机移动能改变画面
- 多物体 transform 正确
- buffer 更新不会破坏 frames in flight

## M4-T6 PBR 与多 Pass 基础

产出：

- metallic-roughness 参数
- normal / albedo / material texture 约定
- shadow 或 post process pass
- HDR / tone mapping 初版

验收标准：

- PBR 参数变化可观察
- pass 输入输出边界明确
- 后续光追 pass 可以读取必要场景数据

## 风险与边界

- 第一版不要把材质系统做成大而全资产框架。
- shader binding 表必须持续同步文档。
- 多 pass 复杂后应进入模块六的 render graph，而不是继续手写顺序。
