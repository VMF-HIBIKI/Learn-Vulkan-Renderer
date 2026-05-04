# M1-S8 Logical Device And Queues 分层

任务：M1-S8 创建 logical device 并获取 graphics/present queue。

```mermaid
graph TD
    Binary["bin: m1_s8_logical_device"] --> PublicApi["run_logical_device_shell"]
    PublicApi --> DeviceModule["device 模块"]
    DeviceModule --> SurfaceModule["surface 模块 / SurfaceBootstrap"]
    DeviceModule --> Selector["select_physical_device"]
    DeviceModule --> LogicalDevice["LogicalDevice"]
    SurfaceModule --> InstanceModule["VulkanInstance"]
    Selector --> PhysicalDevice["VkPhysicalDevice"]
    LogicalDevice --> VkDevice["VkDevice"]
    LogicalDevice --> GraphicsQueue["graphics VkQueue"]
    LogicalDevice --> PresentQueue["present VkQueue"]

    subgraph Project["项目代码"]
        Binary
        PublicApi
        DeviceModule
        SurfaceModule
        InstanceModule
        Selector
        LogicalDevice
    end

    subgraph Vulkan["Vulkan"]
        PhysicalDevice
        VkDevice
        GraphicsQueue
        PresentQueue
    end
```

## 分层说明

| 层级 | 当前职责 | 用到的库 |
| --- | --- | --- |
| device 模块 | 选择 surface-capable GPU，创建 logical device 并获取队列 | `ash` |
| surface 模块 | 提供 instance/surface 查询上下文 | `ash-window` |
| Vulkan 层 | 创建 `VkDevice`，返回 `VkQueue` handles | Vulkan driver |

## 边界

- 本任务只启用 `VK_KHR_swapchain` device extension。
- 本任务不创建 swapchain，不分配 image views，不录制 command buffer。
- `LogicalDevice` 是 `VkDevice` 的 RAII owner，后续 device child object 必须早于它销毁。

