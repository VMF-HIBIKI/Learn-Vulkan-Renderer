# 模块二实现大任务：GPU 内存、资源、上传路径与描述符

导航：
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/02-memory-resource-descriptor.md)
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/02-memory-resource-descriptor.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/02-memory-resource-descriptor.md)

## 大任务总览

| 任务 ID | 名称 | 目标 |
| --- | --- | --- |
| M2-T1 | 内存类型与分配器 | 能根据 usage 和 property 选择 memory type 并绑定资源 |
| M2-T2 | Buffer 系统 | 支持 vertex、index、uniform、storage、staging buffer |
| M2-T3 | Image 系统 | 支持 texture、depth、color target、storage image |
| M2-T4 | 上传路径 | 支持 buffer/image staging upload 和 layout transition |
| M2-T5 | Descriptor 系统 | 支持 layout、pool、set allocation 和 write |
| M2-T6 | Device Address 准备 | 为光追 buffer 和 SBT 提供 device address 能力 |

## M2-T1 内存类型与分配器

产出：

- memory type 查询函数
- `GpuAllocation`
- 最小 device memory allocator

验收标准：

- 能创建 host visible 和 device local allocation
- alignment 与 memory requirements 正确处理
- 销毁顺序可追踪

## M2-T2 Buffer 系统

产出：

- `GpuBuffer`
- buffer usage 封装
- map / unmap / copy helpers

验收标准：

- 能上传顶点数据到 device local buffer
- uniform buffer 可被每帧更新
- storage buffer 可供后续光追和场景数据使用

## M2-T3 Image 系统

产出：

- `GpuImage`
- image view 创建
- sampler 创建
- depth / color / sampled / storage usage

验收标准：

- 能创建 depth image
- 能上传 texture 并采样
- storage image 可作为光追输出目标

## M2-T4 上传路径

产出：

- staging buffer
- immediate submit copy
- buffer-to-image copy
- upload 后资源状态转换

验收标准：

- mesh 和 texture 上传后可被渲染读取
- 上传命令结束后 staging 资源可安全释放或复用
- 验证层不报告 layout / access 错误

## M2-T5 Descriptor 系统

产出：

- descriptor set layout cache
- descriptor pool allocator
- descriptor write helpers
- per-frame descriptor 重置策略

验收标准：

- uniform buffer、combined image sampler、storage image 都能绑定
- descriptor set 不引用已销毁资源
- layout 与 shader binding 可核对

## M2-T6 Device Address 准备

产出：

- buffer device address feature 检查
- device address buffer 创建路径
- address 查询 helper

验收标准：

- 用于 BLAS、TLAS、SBT 的 buffer 能拿到 device address
- usage flags 缺失时能在创建阶段报错
- 文档记录 device address 的安全边界

## 风险与边界

- 第一版 allocator 不要做复杂 suballocation，先确保正确。
- descriptor 不要过早 bindless 化。
- image layout tracking 如果做不完整，至少必须在 pass 文档中显式记录。
