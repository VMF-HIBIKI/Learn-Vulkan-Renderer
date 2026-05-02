# 模块二可执行子任务：GPU 内存、资源、上传路径与描述符

导航：
- [第三层实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/02-memory-resource-descriptor.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/02-memory-resource-descriptor.md)

## 子任务表

| 子任务 ID | 对应大任务 | 子任务内容 | 完成标志 |
| --- | --- | --- | --- |
| M2-S1 | M2-T1 | 查询 physical device memory properties | 能打印 heap/type 信息 |
| M2-S2 | M2-T1 | 实现 memory type selection | 能按 property 选 type |
| M2-S3 | M2-T1 | 定义 `GpuAllocation` 与销毁路径 | allocation 生命周期清晰 |
| M2-S4 | M2-T2 | 创建 buffer 并绑定 memory | buffer 可创建销毁 |
| M2-S5 | M2-T2 | 实现 host visible buffer map/write | CPU 可写入 buffer |
| M2-S6 | M2-T2 | 创建 device local vertex/index buffer | mesh 数据可上传 |
| M2-S7 | M2-T3 | 创建 image、image view 和 sampler | texture 资源可创建 |
| M2-S8 | M2-T3 | 创建 depth image 和 storage image | depth/RT 输出资源可用 |
| M2-S9 | M2-T4 | 实现 staging buffer copy 到 buffer | 顶点数据上传成功 |
| M2-S10 | M2-T4 | 实现 staging buffer copy 到 image | texture 上传成功 |
| M2-S11 | M2-T5 | 创建 descriptor set layout cache | layout 可复用 |
| M2-S12 | M2-T5 | 实现 descriptor pool 和 set allocation | set 可分配和重置 |
| M2-S13 | M2-T5 | 编写 descriptor write helpers | buffer/image 可绑定 |
| M2-S14 | M2-T6 | 启用并查询 buffer device address | address 可用于光追 |

## 执行提示

- M2-S1 到 M2-S3 只处理内存选择，不要急着上传资源。
- M2-S4 到 M2-S10 建立资源与上传闭环。
- M2-S11 到 M2-S14 建立 shader 可见性和光追前置能力。
