# M1-S10 Swapchain Image Views 时序图

```mermaid
sequenceDiagram
    participant Main as m1_s10_swapchain::main
    participant Shell as SwapchainShell
    participant Surface as SurfaceBootstrap
    participant Device as LogicalDevice
    participant Swap as SwapchainBundle
    participant Vulkan as Vulkan

    Main->>Shell: run_swapchain_shell(config)
    Shell->>Surface: SurfaceBootstrap::new(window)
    Shell->>Device: create_logical_device(selected)
    Shell->>Swap: create_swapchain_bundle(surface, device, selected)
    Swap->>Vulkan: vkCreateSwapchainKHR
    Vulkan-->>Swap: VkSwapchainKHR
    Swap->>Vulkan: vkGetSwapchainImagesKHR
    Vulkan-->>Swap: VkImage[]
    loop 每个 swapchain image
        Swap->>Vulkan: vkCreateImageView
        Vulkan-->>Swap: VkImageView
    end
    Shell->>Vulkan: vkDestroyImageView[]
    Shell->>Vulkan: vkDestroySwapchainKHR
    Shell->>Vulkan: vkDestroyDevice
```

## 关键顺序

1. swapchain 创建需要 surface、logical device、queue family indices 和 S9 配置。
2. swapchain images 由 swapchain 拥有；项目只为它们创建 image views。
3. 销毁时 image views 先于 swapchain，swapchain 先于 logical device。

