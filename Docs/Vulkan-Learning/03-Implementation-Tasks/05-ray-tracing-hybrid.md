# 模块五实现大任务：光线追踪、加速结构、SBT 与混合渲染

导航：
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/05-ray-tracing-hybrid.md)
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/05-ray-tracing-hybrid.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/05-ray-tracing-hybrid.md)

## 大任务总览

| 任务 ID | 名称 | 目标 |
| --- | --- | --- |
| M5-T1 | 光追能力查询 | 检查扩展、feature、property 和设备限制 |
| M5-T2 | BLAS 构建 | 将 mesh 几何构建为 bottom-level acceleration structure |
| M5-T3 | TLAS 构建 | 将场景 instance 构建为 top-level acceleration structure |
| M5-T4 | Ray Tracing Pipeline | 创建 raygen、miss、hit shader 和 pipeline |
| M5-T5 | Shader Binding Table | 构建满足对齐要求的 SBT buffer |
| M5-T6 | 混合渲染 Pass | 输出光追阴影、反射或 AO 并与光栅结果合成 |

## M5-T1 光追能力查询

产出：

- extension 列表检查
- feature chain 查询
- ray tracing properties 查询
- 能力日志

验收标准：

- 不支持光追时能优雅降级
- 支持时能打印 SBT 对齐、handle size、max recursion depth
- feature 开启路径与 device 创建集成

## M5-T2 BLAS 构建

产出：

- geometry build input
- acceleration structure buffer
- scratch buffer
- build command

验收标准：

- 简单三角形 mesh 可构建 BLAS
- build 后 BLAS backing buffer 生命周期正确
- 验证层无 AS build 错误

## M5-T3 TLAS 构建

产出：

- instance buffer
- TLAS build input
- transform / instance id / hit group offset
- TLAS update 或 rebuild 策略

验收标准：

- 一个或多个 BLAS instance 可构建 TLAS
- transform 变化后 TLAS 可更新或重建
- TLAS trace 时引用的 BLAS 有效

## M5-T4 Ray Tracing Pipeline

产出：

- raygen shader
- miss shader
- closest hit shader
- shader group 定义
- ray tracing pipeline layout

验收标准：

- pipeline 创建成功
- shader group 数量与 SBT 规划一致
- descriptor 能绑定 TLAS、输出 image 和场景 buffer

## M5-T5 Shader Binding Table

产出：

- shader group handle 查询
- raygen / miss / hit region
- SBT buffer
- device address region

验收标准：

- SBT address、stride、size 满足设备对齐
- raygen、miss、hit region 内容正确
- `vkCmdTraceRaysKHR` 参数可验证

## M5-T6 混合渲染 Pass

产出：

- storage image 输出
- trace rays command
- fullscreen composite pass
- ray traced shadow / reflection / AO 初版

验收标准：

- 光追 pass 能写出可见结果
- 合成 pass 能与光栅画面混合
- RenderDoc 中能看到光追输出纹理和 pass 顺序

## 风险与边界

- 光追要求模块二、三成熟；资源和同步未稳定前不要硬上。
- 第一版递归深度保持最小。
- payload 和 hit attribute 尽量小，后续再做复杂材质。
