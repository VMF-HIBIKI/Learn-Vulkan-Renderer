# M1-S4 Vulkan Validation Logging 时序图

```mermaid
sequenceDiagram
    participant Main as m1_s4_validation::main
    participant Api as run_validation_shell
    participant App as InstanceShell
    participant Entry as ash::Entry
    participant Vulkan as Vulkan Loader/Driver
    participant Debug as VK_EXT_debug_utils
    participant Callback as vulkan_debug_callback

    Main->>Api: run_validation_shell(config)
    Api->>App: InstanceShell::new(config, M1-S4)
    App->>Entry: Entry::load()
    App->>Vulkan: enumerate_instance_layer_properties()
    Vulkan-->>App: layer list
    App->>Vulkan: enumerate_instance_extension_properties()
    Vulkan-->>App: extension list
    App->>Vulkan: vkCreateInstance(layers, extensions, debug pNext)
    Vulkan-->>App: VkInstance
    App->>Debug: create_debug_utils_messenger()
    Debug-->>App: VkDebugUtilsMessengerEXT
    App->>Debug: submit_debug_utils_message()
    Debug->>Callback: print startup diagnostic
    App->>Debug: destroy_debug_utils_messenger()
    App->>Vulkan: vkDestroyInstance()
```

## 关键顺序

1. 先查询 layer/extension 是否存在，再决定是否启用 validation。
2. 创建 instance 时同时把 debug messenger create info 放入 `pNext`，覆盖 instance 创建期间的验证输出。
3. `VkDebugUtilsMessengerEXT` 是 instance child object，销毁顺序必须早于 `VkInstance`。

