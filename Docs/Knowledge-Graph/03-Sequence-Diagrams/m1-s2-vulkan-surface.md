# M1-S2 Vulkan Surface 时序图

```mermaid
sequenceDiagram
    participant Main as m1_s2_surface::main
    participant Api as run_surface_shell
    participant EventLoop as winit::EventLoop
    participant App as SurfaceShell
    participant Active as ActiveEventLoop
    participant Window as winit::Window
    participant Ash as ash
    participant AshWindow as ash-window
    participant Vulkan as Vulkan Loader/Driver

    Main->>Api: run_surface_shell(config)
    Api->>EventLoop: EventLoop::new()
    Api->>App: SurfaceShell::new(config)
    Api->>EventLoop: run_app(&mut app)
    EventLoop->>App: resumed(&ActiveEventLoop)
    App->>Active: create_window(config.attributes())
    Active-->>App: Window
    App->>Window: display_handle()
    App->>Window: window_handle()
    App->>Ash: Entry::load()
    App->>AshWindow: enumerate_required_extensions(display)
    App->>Ash: create_instance(required_extensions)
    Ash->>Vulkan: vkCreateInstance
    Vulkan-->>Ash: VkInstance
    App->>AshWindow: create_surface(entry, instance, display, window)
    AshWindow->>Vulkan: vkCreateWin32SurfaceKHR / platform equivalent
    Vulkan-->>AshWindow: VkSurfaceKHR
    AshWindow-->>App: SurfaceBootstrap
    EventLoop->>App: window_event(CloseRequested)
    App->>Active: exit()
    EventLoop->>App: exiting()
    App->>Vulkan: vkDestroySurfaceKHR
    App->>Vulkan: vkDestroyInstance
```

## 关键顺序

1. 必须先通过 display handle 查询 surface 所需 instance extensions。
2. 必须启用这些 extensions 后创建 `VkInstance`。
3. 必须用同一个 live window/display handle 创建 `VkSurfaceKHR`。
4. 退出时必须先销毁 surface，再销毁 instance。

