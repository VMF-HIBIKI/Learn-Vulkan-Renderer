# 模块三实现大任务：命令录制、同步模型与帧循环

导航：
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/03-command-sync-frame.md)
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/03-command-sync-frame.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/03-command-sync-frame.md)

## 大任务总览

| 任务 ID | 名称 | 目标 |
| --- | --- | --- |
| M3-T1 | Command Pool 与 Recorder | 建立可复用的命令录制入口 |
| M3-T2 | FrameContext | 支持多帧 flight 和 per-frame 资源 |
| M3-T3 | Swapchain 同步 | 正确连接 acquire、submit、present |
| M3-T4 | Barrier 与 Layout Transition | 建立显式资源状态转换工具 |
| M3-T5 | Immediate Submit | 支持上传、一次性 copy 和构建命令 |
| M3-T6 | 延迟销毁 | 解决跨帧 GPU 使用中的资源释放 |

## M3-T1 Command Pool 与 Recorder

产出：

- command pool 创建
- primary command buffer 分配
- begin/end recorder helper

验收标准：

- 每帧可重置并重新录制 command buffer
- recorder 生命周期不会跨越 command buffer end
- Vulkan 调用错误能返回而不是静默吞掉

## M3-T2 FrameContext

产出：

- frame slot 数组
- per-frame command buffer
- per-frame fence / semaphore
- frame index 轮转

验收标准：

- 至少支持 2 frames in flight
- CPU 不会覆盖 GPU 仍在使用的 command buffer
- resize 和关闭时能等待正确 fence

## M3-T3 Swapchain 同步

产出：

- acquire image semaphore
- render finished semaphore
- queue submit
- present

验收标准：

- present 等待 render finished
- submit 等待 image available
- swapchain out-of-date 能触发重建

## M3-T4 Barrier 与 Layout Transition

产出：

- image barrier helper
- buffer barrier helper
- 常见 layout transition 表

验收标准：

- color attachment、depth、sampled image、storage image 转换可验证
- barrier 参数能在日志或文档中解释
- 验证层不报同步和 layout 错误

## M3-T5 Immediate Submit

产出：

- 独立 command pool / command buffer
- 独立 fence
- submit-and-wait helper

验收标准：

- 可用于 staging copy
- 可用于一次性 layout transition
- 等待后临时资源可安全释放

## M3-T6 延迟销毁

产出：

- per-frame deferred deletion queue
- resource release callback
- frame fence 完成后 flush

验收标准：

- resize 时旧 swapchain image view 不会提前销毁
- buffer/image 可延迟释放
- 关闭时能 flush 所有 pending 资源

## 风险与边界

- 同步 helper 不能隐藏语义，必须能追溯 stage/access/layout。
- 第一版可以只用 graphics queue。
- timeline semaphore 和多队列等到资源上传或 render graph 需要时再引入。
