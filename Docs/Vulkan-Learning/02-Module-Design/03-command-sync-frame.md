# 模块三：命令录制、同步模型与帧循环

导航：
- [模块设计索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/README.md)
- [本模块实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/03-command-sync-frame.md)
- [本模块可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/03-command-sync-frame.md)
- [本模块开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/03-command-sync-frame.md)

## 模块职责

这个模块负责把 CPU 侧渲染意图变成 GPU 可执行命令，并确保跨帧、跨队列、跨资源访问安全：

- command pool / command buffer
- fence / binary semaphore / timeline semaphore
- image acquire / queue submit / present
- buffer barrier / image barrier
- layout transition
- frame-in-flight
- immediate submit
- deferred destruction

## 关键设计问题

### 1. 同步模型如何学习

不要把 barrier 当作“玄学修复”。每次 barrier 都要能回答：

- 前一个访问是什么
- 后一个访问是什么
- 哪个 pipeline stage 产生数据
- 哪个 pipeline stage 消费数据
- layout 是否变化
- queue family 是否转移

### 2. FrameContext 应该拥有多少东西

每个 frame slot 建议拥有：

- command pool / command buffer
- render finished semaphore
- image available semaphore
- in-flight fence
- per-frame descriptor pool
- 延迟销毁队列

### 3. Immediate submit 是否保留

保留。它适合资源上传、一次性 layout transition、BLAS build smoke test。但要清楚它通常会等待 GPU，不能滥用到热路径。

### 4. Timeline semaphore 何时引入

第一版可以先用 fence + binary semaphore。等多队列、异步上传或复杂 render graph 出现后，再引入 timeline semaphore 简化依赖追踪。

## 建议数据结构

- `FrameContext`
  - frame index
  - command pool
  - command buffer
  - fence
  - semaphores
  - deferred releases
- `CommandRecorder`
  - active command buffer
  - debug label stack
  - resource state access
- `SubmitBatch`
  - wait semaphores
  - signal semaphores
  - command buffers
  - fence
- `ResourceBarrier`
  - resource handle
  - old/new access
  - old/new layout
  - src/dst stage

## 模块接口建议

- `begin_frame() -> FrameToken`
- `begin_commands(frame) -> CommandRecorder`
- `transition_image(cmd, image, old, new)`
- `copy_buffer(cmd, src, dst, regions)`
- `submit_graphics(batch)`
- `present(frame, image_index)`
- `defer_destroy(frame, resource)`

## 关键不变量

- command buffer reset 前必须确认 GPU 不再使用。
- frame-local 资源至少延迟到对应 fence 完成后销毁。
- acquire semaphore 必须被提交等待，render finished semaphore 必须被 present 等待。
- swapchain image layout 在 present 前必须转换到 present layout。
- 同一资源的读写冲突必须通过 barrier 或 pass 顺序解决。

## 与其他模块的耦合点

- 使用模块一提供的 queue 和 swapchain。
- 使用模块二提供的资源状态和 copy/upload 信息。
- 为模块四和模块五提供 pass command recording 能力。
- 为模块六的 render graph 提供底层同步执行接口。

## 参考资料

- Vulkan Guide：synchronization、swapchain、queue submit。
- Khronos Tutorial：drawing a triangle、frames in flight。
- RenderDoc：检查实际 image layout 和 pass 顺序。

## 设计取舍

- 第一版同步宁可保守，也不要跳过理由。
- 复杂 barrier 应先写成显式辅助函数，不要埋在 pass 内部。
- 同步抽象必须能解释到底层 Vulkan stage / access，否则后期很难调试。
