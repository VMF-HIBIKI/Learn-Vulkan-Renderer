# 模块一可执行子任务：Vulkan 平台、实例、物理设备与交换链

导航：
- [第三层实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/01-platform-device-swapchain.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/01-platform-device-swapchain.md)

## 子任务表

| 子任务 ID | 对应大任务 | 子任务内容 | 完成标志 |
| --- | --- | --- | --- |
| M1-S1 | M1-T1 | 选择窗口库并创建最小窗口 | 能打开和关闭窗口 |
| M1-S2 | M1-T1 | 接入 raw window/display handle 并创建 surface | surface 创建成功 |
| M1-S3 | M1-T2 | 创建 Vulkan entry 和 instance | instance 创建成功 |
| M1-S4 | M1-T2 | 开启 validation layer 和 debug messenger | 能打印验证层消息 |
| M1-S5 | M1-T3 | 枚举 physical devices 并打印 properties | 日志列出 GPU 信息 |
| M1-S6 | M1-T3 | 查询 queue family 和 swapchain 支持 | 能找到 graphics/present queue |
| M1-S7 | M1-T3 | 检查 device extension 和光追 feature | 能记录支持矩阵 |
| M1-S8 | M1-T4 | 创建 logical device 并获取队列 | device/queue 可用 |
| M1-S9 | M1-T5 | 选择 surface format、present mode、extent | swapchain 参数稳定 |
| M1-S10 | M1-T5 | 创建 swapchain 和 image views | image view 数量正确 |
| M1-S11 | M1-T5 | 实现 resize 标记与 swapchain recreate | resize 后继续运行 |
| M1-S12 | M1-T6 | 录制 clear swapchain image 的命令 | 屏幕出现清屏颜色 |
| M1-S13 | M1-T6 | 整合 acquire / submit / present | 帧循环稳定呈现 |

## 执行提示

- M1-S1 到 M1-S4 只解决启动和日志，不要提前创建 swapchain。
- M1-S5 到 M1-S8 只解决设备选择，不要混入资源系统。
- M1-S9 到 M1-S13 的重点是稳定呈现，不是画复杂几何。
