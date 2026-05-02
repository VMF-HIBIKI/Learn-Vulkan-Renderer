# 第二层：模块设计

返回上层：
- [Vulkan Learning 根索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/README.md)
- [第一层索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/01-Macro-Overview/README.md)

## 六个模块

1. [Vulkan 平台、实例、物理设备与交换链](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/01-platform-device-swapchain.md)
2. [GPU 内存、资源、上传路径与描述符](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/02-memory-resource-descriptor.md)
3. [命令录制、同步模型与帧循环](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/03-command-sync-frame.md)
4. [光栅管线、着色器、材质与场景绘制](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/04-raster-pipeline-scene.md)
5. [光线追踪、加速结构、SBT 与混合渲染](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/05-ray-tracing-hybrid.md)
6. [渲染图、调试工具、性能分析与工程化](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/06-render-graph-tooling.md)

## 本层目标

- 明确每个模块的职责、边界和内部数据结构。
- 明确每个模块和 Vulkan / ash / Khronos samples 中哪些概念相互对应。
- 为第三层的大任务拆分准备稳定接口。

## 下钻入口

- [第三层实现大任务索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/README.md)
