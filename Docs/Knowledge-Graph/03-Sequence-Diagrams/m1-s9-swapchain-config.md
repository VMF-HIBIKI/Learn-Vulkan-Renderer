# M1-S9 Swapchain Configuration 时序图

```mermaid
sequenceDiagram
    participant Main as m1_s9_swapchain_config::main
    participant Shell as SwapchainConfigShell
    participant Surface as SurfaceBootstrap
    participant Device as device 模块
    participant Swap as swapchain 模块
    participant Vulkan as Vulkan WSI

    Main->>Shell: run_swapchain_config_shell(config)
    Shell->>Surface: SurfaceBootstrap::new(window)
    Shell->>Device: select_physical_device(instance, surface)
    Device-->>Shell: SelectedPhysicalDevice
    Shell->>Swap: query_swapchain_support_details(device, surface)
    Swap->>Vulkan: capabilities/formats/present_modes
    Vulkan-->>Swap: SwapchainSupportDetails
    Shell->>Swap: choose_swapchain_config(details, window_size)
    Swap-->>Shell: SwapchainConfig
    Shell->>Shell: println selected config
```

## 关键顺序

1. 必须先有 selected physical device 和 surface，才能查询 swapchain support。
2. 参数选择是纯策略，不创建 Vulkan 对象。
3. S10 会使用这里的 `SwapchainConfig` 创建真正的 swapchain 和 image views。

