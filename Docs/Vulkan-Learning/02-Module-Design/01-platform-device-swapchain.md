# 模块一：Vulkan 平台、实例、物理设备与交换链

导航：
- [模块设计索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/README.md)
- [本模块实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/01-platform-device-swapchain.md)
- [本模块可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/01-platform-device-swapchain.md)
- [本模块开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/01-platform-device-swapchain.md)

## 模块职责

这个模块负责 Vulkan 程序能否启动、选择设备并稳定呈现：

- window 与 Vulkan surface
- Vulkan entry、instance 与 debug messenger
- physical device 枚举与能力评分
- queue family 选择
- logical device 与队列获取
- swapchain、swapchain image view 与重建流程

如果这一层不稳定，后面的资源系统、命令录制和渲染 pass 都没有可靠执行环境。

## 关键设计问题

### 1. 设备选择标准是什么

第一版不要只选第一个 GPU。至少要检查：

- graphics queue
- present queue
- swapchain 支持
- 必要 device extension
- 后续光追阶段需要的 feature 是否可选开启

### 2. Surface 和 swapchain 谁拥有

`Surface` 依赖 window 生命周期，`Swapchain` 依赖 surface、device 和窗口尺寸。建议让 platform 层拥有 window，让 renderer backend 拥有 surface / swapchain，并在 resize 时重建 swapchain 相关资源。

### 3. Debug messenger 是否默认开启

开发期默认开启。验证层输出是 Vulkan 学习阶段最重要的反馈渠道之一。

### 4. Swapchain 重建如何设计

swapchain image、image view、framebuffer 或 render target 都会受重建影响。第一版可以直接 `device_wait_idle` 后重建，后续再优化为按 frame 安全回收。

## 建议数据结构

- `WindowState`
  - window handle
  - framebuffer size
  - resize flag
- `VulkanInstance`
  - `ash::Entry`
  - `ash::Instance`
  - debug messenger
- `PhysicalDeviceInfo`
  - handle
  - properties
  - features
  - queue families
  - extension support
- `DeviceContext`
  - logical device
  - graphics queue
  - present queue
  - queue family indices
- `SwapchainState`
  - swapchain handle
  - format
  - extent
  - images
  - image views

## 模块接口建议

- `VulkanContext::new(window) -> Result<Self>`
- `VulkanContext::device() -> &ash::Device`
- `VulkanContext::queues() -> QueueHandles`
- `Swapchain::new(context, surface, size) -> Result<Self>`
- `Swapchain::recreate(context, size) -> Result<()>`
- `Swapchain::acquire_next_image(frame) -> AcquireResult`

## 关键不变量

- instance 必须活得比 surface、device 和 debug messenger 更久。
- device 必须活得比 swapchain、资源、pipeline 和 command pool 更久。
- swapchain image view 不能在 GPU 仍使用时销毁。
- resize 后旧 swapchain 相关资源必须进入延迟销毁或等待 GPU idle。

## 与其他模块的耦合点

- 向模块二提供 device、physical device memory properties 和 extension capability。
- 向模块三提供 queue、swapchain acquire / present 信息。
- 向模块四提供 swapchain format、extent 和 render target。
- 向模块五提供 ray tracing feature 查询基础。

## 参考资料

- Khronos Vulkan Tutorial：instance、device、swapchain 章节。
- Vulkan Guide：window system integration、debug utils、swapchain。
- Ash examples：entry / instance / extension loader 使用方式。

## 设计取舍

- 第一版优先清晰，不做复杂多 GPU。
- 第一版可以用 `device_wait_idle` 简化 swapchain 重建。
- 光追 feature 先检测和记录，不必在第一阶段强制要求开启。
