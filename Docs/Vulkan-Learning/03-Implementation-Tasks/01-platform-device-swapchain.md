# 模块一实现大任务：Vulkan 平台、实例、物理设备与交换链

导航：
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/01-platform-device-swapchain.md)
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/01-platform-device-swapchain.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/01-platform-device-swapchain.md)

## 大任务总览

| 任务 ID | 名称 | 目标 |
| --- | --- | --- |
| M1-T1 | 窗口与 Surface | 建立 window handle 到 Vulkan surface 的连接 |
| M1-T2 | Instance 与 Debug Utils | 创建 Vulkan instance 并接入验证层输出 |
| M1-T3 | Physical Device 选择 | 枚举 GPU、检查队列、扩展和基础 feature |
| M1-T4 | Logical Device 与队列 | 创建 device 并获取 graphics / present queue |
| M1-T5 | Swapchain 创建与重建 | 创建 swapchain、image view，并处理 resize |
| M1-T6 | 清屏呈现闭环 | 不依赖复杂 pipeline，先稳定 clear 到屏幕 |

## M1-T1 窗口与 Surface

产出：

- window 创建入口
- raw window/display handle 获取路径
- Vulkan surface 创建封装

验收标准：

- 程序能打开窗口
- surface 创建失败时有明确错误
- window 关闭能正常释放 surface

## M1-T2 Instance 与 Debug Utils

产出：

- `VulkanInstance`
- validation layer 开关
- debug messenger callback

验收标准：

- 开发模式能收到验证层日志
- release 或配置关闭时不强依赖验证层
- instance 销毁顺序正确

## M1-T3 Physical Device 选择

产出：

- GPU 枚举
- queue family 查询
- swapchain extension 检查
- device suitability 评分

验收标准：

- 没有合适 GPU 时给出可读错误
- 能打印选中的 GPU 名称、类型和关键 feature
- 能记录光追扩展是否支持

## M1-T4 Logical Device 与队列

产出：

- logical device 创建
- graphics queue / present queue 获取
- device extension 启用列表

验收标准：

- device 创建后能获取所需队列
- 队列 family index 与物理设备查询一致
- device 销毁晚于所有 device child object

## M1-T5 Swapchain 创建与重建

产出：

- surface format / present mode / extent 选择
- swapchain images
- image views
- resize recreate 流程

验收标准：

- 窗口 resize 后不崩溃
- swapchain image view 数量与 image 数量一致
- 旧 swapchain 资源不会提前销毁

## M1-T6 清屏呈现闭环

产出：

- acquire / submit / present 的最小路径
- clear color command
- resize 后继续呈现

验收标准：

- 能稳定显示清屏颜色
- 验证层无错误
- 关闭窗口时无 device lost 和资源泄漏日志

## 风险与边界

- 这一模块不要急着写 graphics pipeline。
- swapchain resize 先保守处理，别过早优化。
- 光追能力只查询和记录，不在第一模块强制启用。
