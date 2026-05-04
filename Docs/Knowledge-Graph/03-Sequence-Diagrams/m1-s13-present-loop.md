# M1-S13 Acquire Submit Present Loop 时序图

```mermaid
sequenceDiagram
    participant Winit as winit
    participant Shell as PresentLoopShell
    participant Sync as FrameSync
    participant Swap as SwapchainBundle
    participant Device as LogicalDevice
    participant Vulkan as Vulkan

    Winit->>Shell: about_to_wait()
    Shell->>Winit: request_redraw()
    Winit->>Shell: WindowEvent::RedrawRequested
    Shell->>Sync: wait in_flight fence
    Sync->>Vulkan: vkWaitForFences
    Shell->>Swap: acquire_next_image(image_available)
    Swap->>Vulkan: vkAcquireNextImageKHR
    Vulkan-->>Shell: image_index
    Shell->>Sync: reset in_flight fence
    Sync->>Vulkan: vkResetFences
    Shell->>Device: queue_submit(command_buffer[image_index])
    Device->>Vulkan: vkQueueSubmit(wait image_available, signal render_finished)
    Shell->>Swap: queue_present(render_finished, image_index)
    Swap->>Vulkan: vkQueuePresentKHR
    alt out-of-date or suboptimal
        Shell->>Device: wait_idle()
        Shell->>Shell: recreate swapchain and commands
    end
```

## 关键顺序

1. acquire 等待 image-available semaphore。
2. submit 等待 image-available，执行对应 image 的 command buffer，并 signal render-finished。
3. present 等待 render-finished。
4. resize/out-of-date 触发 swapchain 与 command buffers 一起重建。

