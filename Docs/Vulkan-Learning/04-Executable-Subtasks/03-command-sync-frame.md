# 模块三可执行子任务：命令录制、同步模型与帧循环

导航：
- [第三层实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/03-command-sync-frame.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/03-command-sync-frame.md)

## 子任务表

| 子任务 ID | 对应大任务 | 子任务内容 | 完成标志 |
| --- | --- | --- | --- |
| M3-S1 | M3-T1 | 创建 command pool | pool 与 queue family 匹配 |
| M3-S2 | M3-T1 | 分配 primary command buffer | command buffer 可录制 |
| M3-S3 | M3-T1 | 封装 begin/end recorder | 录制流程可复用 |
| M3-S4 | M3-T2 | 定义 `FrameContext` | frame-local 资源集中 |
| M3-S5 | M3-T2 | 创建 fence 和 semaphores | 同步对象可用 |
| M3-S6 | M3-T2 | 实现 frames-in-flight 轮转 | CPU 不覆盖 GPU 使用中资源 |
| M3-S7 | M3-T3 | 整合 acquire image | 能处理 image index |
| M3-S8 | M3-T3 | 整合 queue submit | wait/signal 正确 |
| M3-S9 | M3-T3 | 整合 present 和 out-of-date 处理 | resize 路径可触发 |
| M3-S10 | M3-T4 | 实现 image layout transition helper | 常见 layout 可转换 |
| M3-S11 | M3-T4 | 实现 buffer/image barrier helper | 读写依赖可表达 |
| M3-S12 | M3-T5 | 实现 immediate submit | 上传命令可一次性提交 |
| M3-S13 | M3-T6 | 实现 per-frame deferred destruction | 资源延迟释放安全 |

## 执行提示

- M3-S1 到 M3-S6 先让帧资源有明确生命周期。
- M3-S7 到 M3-S9 专注 swapchain 同步。
- M3-S10 到 M3-S13 是后续资源上传、光栅 pass 和光追 build 的地基。
