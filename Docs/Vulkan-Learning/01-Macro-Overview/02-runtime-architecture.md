# 渲染器运行时架构全景

导航：
- [第一层索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/01-Macro-Overview/README.md)
- [第二层模块设计](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/README.md)

## 总体数据流

一个 Vulkan 渲染器的运行时可以按数据流理解：

```text
Window
  -> Surface / Swapchain
  -> Device / Queues
  -> Resource Allocator
  -> Descriptor / Pipeline Layout
  -> Command Recording
  -> Queue Submit
  -> Present
```

光栅与光追不是两套孤立系统。它们共享：

- device、queue、command buffer
- buffer、image、sampler、descriptor
- shader 编译与 pipeline cache
- synchronization 与 render graph
- scene、material、camera 与 GPU 参数

区别在于光追多出 acceleration structure、shader binding table、ray tracing pipeline 和更严格的 buffer device address 使用。

## 分层架构

### 1. Platform 层

负责窗口、surface、swapchain 和输入事件。

这层的目标是把平台差异挡在外面，让渲染器核心只看到 frame acquire / present 的接口。

### 2. Vulkan Backend 层

负责 Vulkan handle 的创建、销毁和安全封装：

- instance / debug messenger
- physical device / logical device
- queues / command pool
- swapchain / image views
- allocator / descriptor allocator
- pipeline factory / shader module

这层允许 `unsafe`，但必须把安全不变量写清楚。

### 3. Renderer Core 层

负责渲染概念：

- frame context
- render target
- mesh / material / texture
- pass
- render graph
- scene GPU data

这层应尽量保持 safe Rust，不直接暴露裸 Vulkan handle。

### 4. Technique 层

负责具体渲染算法：

- first triangle
- depth prepass
- forward shading
- deferred shading
- shadow map
- ray traced shadow
- ray traced reflection
- path tracing preview

算法层只声明需要哪些资源和 pass，不直接管理全局生命周期。

### 5. Tooling 层

负责调试和可观测性：

- validation layer 开关
- debug name
- GPU timestamp
- RenderDoc 捕获约定
- shader 编译日志
- pipeline cache 统计
- 资源泄漏检查

## 帧循环心智模型

每帧不是“调用 render 一下”，而是一个明确状态机：

1. 处理窗口事件。
2. acquire swapchain image。
3. 等待当前 frame slot 的 fence。
4. 回收上一轮 frame-local 资源。
5. 更新 CPU 侧 scene 数据。
6. 上传变更资源。
7. 构建或更新渲染图。
8. 录制 command buffer。
9. submit 到 graphics / compute 队列。
10. present swapchain image。

## 关键所有权边界

- `VulkanContext` 拥有 instance、device、queues 和全局函数入口。
- `Swapchain` 拥有 swapchain images 对应的 image views，但不拥有原始 swapchain image 内存。
- `Allocator` 拥有 memory allocation；buffer / image 只持有分配句柄。
- `FrameContext` 拥有 per-frame command buffer、fence、semaphore 和延迟销毁队列。
- `Renderer` 拥有资源表、渲染图和技术 pass。

## 光追加入后的架构变化

加入光追后，渲染器必须新增几条数据路径：

- Mesh buffer 必须可用于 acceleration structure build。
- Material 和 instance 数据必须可被 hit shader 访问。
- BLAS/TLAS 构建需要 scratch buffer 和 build/update 策略。
- SBT 需要按 shader group handle 对齐并上传到 device address buffer。
- 光追输出通常写入 storage image，再被后续 pass 合成到 swapchain。

这要求第二模块和第三模块先打牢，否则第五模块会在资源和同步上失控。
