# M1-S12 Clear Command Recording 时序图

```mermaid
sequenceDiagram
    participant Shell as ClearCommandShell
    participant Device as LogicalDevice
    participant Swap as SwapchainBundle
    participant Cmd as ClearCommandBundle
    participant Vulkan as Vulkan

    Shell->>Swap: create_swapchain_bundle()
    Shell->>Cmd: create_clear_command_bundle(device, swapchain)
    Cmd->>Vulkan: vkCreateCommandPool(graphics_family)
    Cmd->>Vulkan: vkAllocateCommandBuffers(image_count)
    loop 每个 swapchain image
        Cmd->>Vulkan: vkBeginCommandBuffer
        Cmd->>Vulkan: vkCmdPipelineBarrier(UNDEFINED -> TRANSFER_DST)
        Cmd->>Vulkan: vkCmdClearColorImage
        Cmd->>Vulkan: vkCmdPipelineBarrier(TRANSFER_DST -> PRESENT_SRC)
        Cmd->>Vulkan: vkEndCommandBuffer
    end
    Cmd->>Vulkan: vkDestroyCommandPool
```

## 关键顺序

1. command pool 必须绑定 graphics queue family。
2. clear 前先把 swapchain image 转到 `TRANSFER_DST_OPTIMAL`。
3. clear 后转到 `PRESENT_SRC_KHR`，为 S13 present 做准备。

