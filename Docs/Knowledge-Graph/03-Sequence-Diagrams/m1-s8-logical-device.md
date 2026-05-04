# M1-S8 Logical Device And Queues 时序图

```mermaid
sequenceDiagram
    participant Main as m1_s8_logical_device::main
    participant Shell as LogicalDeviceShell
    participant Surface as SurfaceBootstrap
    participant DeviceModule as device 模块
    participant Vulkan as Vulkan Driver

    Main->>Shell: run_logical_device_shell(config)
    Shell->>Surface: SurfaceBootstrap::new(window)
    Surface->>Vulkan: vkCreateInstance + vkCreateSurfaceKHR
    Shell->>DeviceModule: select_physical_device(instance, surface)
    DeviceModule->>Vulkan: enumerate/query devices
    Vulkan-->>DeviceModule: selected physical device
    Shell->>DeviceModule: create_logical_device(selected)
    DeviceModule->>Vulkan: vkCreateDevice(queue infos, VK_KHR_swapchain)
    Vulkan-->>DeviceModule: VkDevice
    DeviceModule->>Vulkan: vkGetDeviceQueue(graphics)
    DeviceModule->>Vulkan: vkGetDeviceQueue(present)
    DeviceModule-->>Shell: LogicalDevice
    Shell->>Vulkan: vkDestroyDevice
    Shell->>Vulkan: vkDestroySurfaceKHR
    Shell->>Vulkan: vkDestroyInstance
```

## 关键顺序

1. 先选择同时满足 graphics/present/swapchain 的 physical device。
2. 创建 logical device 时必须启用 `VK_KHR_swapchain`。
3. device child objects 后续必须先于 `VkDevice` 销毁。

