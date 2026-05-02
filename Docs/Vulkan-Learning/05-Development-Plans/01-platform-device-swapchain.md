# 模块一开发计划：Vulkan 平台、实例、物理设备与交换链

导航：
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/01-platform-device-swapchain.md)
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/01-platform-device-swapchain.md)

## 推荐开发顺序

| 顺序 | 子任务 | 建议 Issue 标题 | 前置条件 | 测试重点 | 完成定义 |
| --- | --- | --- | --- | --- | --- |
| 1 | M1-S1 | `feat: create desktop window shell` | 无 | 窗口打开/关闭 | 有最小窗口程序 |
| 2 | M1-S2 | `feat: create vulkan surface from window` | M1-S1 | surface 创建失败路径 | surface 生命周期清晰 |
| 3 | M1-S3 | `feat: initialize vulkan instance` | M1-S2 | required extensions | instance 可创建销毁 |
| 4 | M1-S4 | `feat: enable vulkan validation logging` | M1-S3 | debug callback | 验证层消息可见 |
| 5 | M1-S5 | `feat: enumerate vulkan physical devices` | M1-S3 | GPU properties 日志 | 能列出设备信息 |
| 6 | M1-S6 | `feat: select graphics and present queues` | M1-S5 | queue family 支持 | 队列选择稳定 |
| 7 | M1-S7 | `feat: report required device features` | M1-S5 | extension/feature 矩阵 | 光追能力可记录 |
| 8 | M1-S8 | `feat: create logical device and queues` | M1-S6 | device extension | queue handles 可用 |
| 9 | M1-S9 | `feat: choose swapchain configuration` | M1-S8 | format/present/extent | 参数选择可解释 |
| 10 | M1-S10 | `feat: create swapchain image views` | M1-S9 | image view 数量 | swapchain 资源完整 |
| 11 | M1-S11 | `feat: recreate swapchain on resize` | M1-S10 | resize/out-of-date | resize 后稳定 |
| 12 | M1-S12 | `demo: clear swapchain image` | M1-S10 | layout/clear 命令 | 屏幕有清屏颜色 |
| 13 | M1-S13 | `feat: run acquire submit present loop` | M1-S12 | 帧循环同步 | 验证层无错误 |

## 实施建议

- 先让 validation layer 成为默认反馈渠道。
- 每一步都打印关键 Vulkan 对象名称和选择理由。
- swapchain 重建先用保守策略，等模块三延迟销毁成熟后再优化。

## 每个子任务的固定动作

1. 记录这一步新增了哪些 Vulkan handle。
2. 说明这些 handle 的 owner 和销毁顺序。
3. 运行 `cargo fmt --all -- --check`。
4. 运行 `cargo clippy --all-targets --all-features -- -D warnings`。
5. 有运行窗口的任务，开启 validation layer 并确认没有 error。
