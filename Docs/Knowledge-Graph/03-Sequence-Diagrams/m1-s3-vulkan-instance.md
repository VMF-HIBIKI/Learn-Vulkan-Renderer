# M1-S3 Vulkan Instance 时序图

```mermaid
sequenceDiagram
    participant Main as m1_s3_instance::main
    participant Api as run_instance_shell
    participant EventLoop as winit::EventLoop
    participant App as InstanceShell
    participant Active as ActiveEventLoop
    participant Window as winit::Window
    participant AshWindow as ash-window
    participant Ash as ash::Entry
    participant Vulkan as Vulkan Loader/Driver

    Main->>Api: run_instance_shell(config)
    Api->>EventLoop: EventLoop::new()
    Api->>App: InstanceShell::new(config)
    Api->>EventLoop: run_app(&mut app)
    EventLoop->>App: resumed(&ActiveEventLoop)
    App->>Active: create_window(config.attributes())
    Active-->>App: Window
    App->>Window: display_handle()
    App->>Ash: Entry::load()
    App->>AshWindow: enumerate_required_extensions(display)
    AshWindow-->>App: required extension names
    App->>Ash: create_instance(application_info, extensions)
    Ash->>Vulkan: vkCreateInstance
    Vulkan-->>Ash: VkInstance
    App-->>EventLoop: VulkanInstance stored
    EventLoop->>App: window_event(CloseRequested)
    App->>Active: exit()
    EventLoop->>App: exiting()
    App->>Vulkan: vkDestroyInstance
```

## 关键顺序

1. `Entry` 必须先从平台 Vulkan loader 加载。
2. `VkInstance` 创建前必须根据 display handle 启用 surface 相关 instance extensions。
3. `VulkanInstance` 持有 `Entry` 和 `Instance`，保证函数表上下文覆盖 instance 生命周期。
4. `VkInstance` 必须晚于所有 instance child object 销毁；当前 M1-S3 没有 child object，M1-S2 surface 路径由 `SurfaceBootstrap` 先销毁 surface。

