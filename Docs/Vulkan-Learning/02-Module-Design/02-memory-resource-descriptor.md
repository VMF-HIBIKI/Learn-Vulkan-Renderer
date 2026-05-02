# 模块二：GPU 内存、资源、上传路径与描述符

导航：
- [模块设计索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/README.md)
- [本模块实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/02-memory-resource-descriptor.md)
- [本模块可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/02-memory-resource-descriptor.md)
- [本模块开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/02-memory-resource-descriptor.md)

## 模块职责

这个模块负责渲染器中所有 GPU 数据的创建、上传、绑定和 shader 可见性：

- buffer / image / image view / sampler
- device memory 分配与绑定
- staging buffer 上传路径
- descriptor set layout / descriptor pool / descriptor set
- uniform、storage、sampled image、storage image 等资源绑定
- buffer device address，为光追加速结构做准备

## 关键设计问题

### 1. 先手写 allocator 还是直接引入分配库

学习阶段建议先实现一个最小 allocator，理解 memory type、heap、alignment 和 bind 的语义。等资源数量上来后，再评估 `gpu-allocator` 或 Vulkan Memory Allocator 绑定。

### 2. Buffer 和 Image 是否暴露裸 handle

后端层可以持有裸 handle，渲染器核心应通过 `BufferHandle`、`ImageHandle` 或资源表索引访问。这样后续做延迟销毁、资源别名和渲染图会更稳。

### 3. Descriptor 模型如何不过早复杂化

第一版用明确的 set layout 和 per-frame descriptor pool。后续再演进到 bindless、descriptor indexing 或 descriptor buffer。

### 4. 上传路径如何避免阻塞主循环

第一版可以使用 immediate submit。后续引入 upload queue、ring buffer、staging arena 和 copy pass。

## 建议数据结构

- `GpuAllocation`
  - device memory
  - offset
  - size
  - memory type index
- `GpuBuffer`
  - buffer handle
  - allocation
  - size
  - usage flags
  - optional device address
- `GpuImage`
  - image handle
  - allocation
  - format
  - extent
  - mip levels
  - current layout tracking
- `DescriptorAllocator`
  - pools
  - per-frame reset policy
- `DescriptorSetLayoutCache`
  - layout key
  - layout handle
- `UploadContext`
  - staging buffer
  - command buffer
  - fence

## 模块接口建议

- `create_buffer(desc) -> BufferHandle`
- `create_image(desc) -> ImageHandle`
- `upload_buffer_data(handle, bytes)`
- `upload_image_data(handle, pixels, layout)`
- `create_descriptor_layout(desc) -> DescriptorLayoutHandle`
- `allocate_descriptor_set(layout) -> DescriptorSetHandle`
- `write_descriptor_set(writes)`

## 关键不变量

- GPU 资源销毁必须晚于最后一次 GPU 使用。
- host visible memory 的 map / flush / invalidate 规则必须明确。
- image layout 不能靠猜，必须由 pass 或资源状态系统记录。
- descriptor set 引用的资源必须活得比 descriptor set 使用更久。
- buffer device address 资源必须用正确 usage 和 feature 创建。

## 与其他模块的耦合点

- 向模块三提供 command copy 和 barrier 的资源信息。
- 向模块四提供 vertex、index、uniform、texture 和 render target。
- 向模块五提供 acceleration structure buffer、scratch buffer、SBT buffer。
- 向模块六提供资源注册表和生命周期追踪数据。

## 参考资料

- Vulkan Guide：memory allocation、synchronization、descriptor 相关章节。
- Khronos Tutorial：vertex buffer、index buffer、uniform buffer、texture image。
- Vulkan Ray Tracing Guide：acceleration structure buffer 与 device address。

## 设计取舍

- 第一版 allocator 可以保守和低效，但必须正确。
- 第一版 descriptor 可以冗长，但绑定关系必须可读。
- 不要把资源上传逻辑散落到各个 pass 中，统一收敛到 resource / upload 模块。
