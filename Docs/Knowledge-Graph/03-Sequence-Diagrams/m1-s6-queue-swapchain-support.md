# M1-S6 Queue And Swapchain Support 时序图

```mermaid
sequenceDiagram
    participant Main as m1_s6_queue_support::main
    participant Shell as QueueSupportShell
    participant Surface as SurfaceBootstrap
    participant Device as device 模块
    participant Vulkan as Vulkan/WSI

    Main->>Shell: run_queue_support_shell(config)
    Shell->>Surface: SurfaceBootstrap::new(window)
    Surface->>Vulkan: vkCreateInstance + vkCreateSurfaceKHR
    Shell->>Device: enumerate_physical_devices(instance)
    Device->>Vulkan: vkEnumeratePhysicalDevices
    Vulkan-->>Device: VkPhysicalDevice[]
    loop 每个 GPU
        Shell->>Device: query_queue_family_support(device, surface)
        Device->>Vulkan: vkGetPhysicalDeviceQueueFamilyProperties
        Device->>Vulkan: vkGetPhysicalDeviceSurfaceSupportKHR
        Shell->>Device: query_swapchain_support_summary(device, surface)
        Device->>Vulkan: capabilities/formats/present_modes
        Shell->>Shell: println support summary
    end
    Shell->>Vulkan: vkDestroySurfaceKHR
    Shell->>Vulkan: vkDestroyInstance
```

## 关键顺序

1. present queue 支持必须针对具体 `VkSurfaceKHR` 查询。
2. graphics queue 支持来自 queue family flags，present 支持来自 WSI 扩展函数。
3. swapchain 可用性至少需要有 surface format 和 present mode。

