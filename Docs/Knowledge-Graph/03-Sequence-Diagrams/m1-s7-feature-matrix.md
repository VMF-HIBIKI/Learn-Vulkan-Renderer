# M1-S7 Device Extension And Feature Matrix 时序图

```mermaid
sequenceDiagram
    participant Main as m1_s7_feature_matrix::main
    participant Shell as FeatureMatrixShell
    participant Instance as VulkanInstance
    participant Device as device 模块
    participant Vulkan as Vulkan Driver

    Main->>Shell: run_feature_matrix_shell(config)
    Shell->>Instance: new_for_display(display_handle)
    Instance->>Vulkan: vkCreateInstance
    Shell->>Device: enumerate_physical_devices(instance)
    Device->>Vulkan: vkEnumeratePhysicalDevices
    loop 每个 GPU
        Shell->>Device: query_physical_device_feature_matrix(device)
        Device->>Vulkan: vkEnumerateDeviceExtensionProperties
        Vulkan-->>Device: extension list
        Device->>Vulkan: vkGetPhysicalDeviceFeatures2(pNext feature chain)
        Vulkan-->>Device: feature bits
        Device-->>Shell: PhysicalDeviceFeatureMatrix
        Shell->>Shell: println support matrix
    end
    Shell->>Vulkan: vkDestroyInstance
```

## 关键顺序

1. device extension 支持必须逐个 physical device 查询。
2. 光追 feature bits 通过 `vkGetPhysicalDeviceFeatures2` 的 `pNext` 链查询。
3. 本任务只记录能力，不把能力写进 `VkDeviceCreateInfo`。

