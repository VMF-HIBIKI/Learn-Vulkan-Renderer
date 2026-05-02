# 第五层：子任务开发计划

返回上层：
- [Vulkan Learning 根索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/README.md)
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/README.md)

## 开发计划索引

1. [Vulkan 平台、实例、物理设备与交换链](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/01-platform-device-swapchain.md)
2. [GPU 内存、资源、上传路径与描述符](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/02-memory-resource-descriptor.md)
3. [命令录制、同步模型与帧循环](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/03-command-sync-frame.md)
4. [光栅管线、着色器、材质与场景绘制](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/04-raster-pipeline-scene.md)
5. [光线追踪、加速结构、SBT 与混合渲染](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/05-ray-tracing-hybrid.md)
6. [渲染图、调试工具、性能分析与工程化](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/06-render-graph-tooling.md)

## 本层目标

- 为每个子任务给出推荐执行顺序、Issue 标题、前置条件、测试重点和完成定义。
- 让学习过程可追踪、可暂停、可恢复。
- 用小步提交迫使每一步都能解释自己的 Vulkan 语义。

## 每个子任务的固定动作

1. 先写最小 smoke test、运行记录或验证层验收方式。
2. 再写实现，不要跳过资源所有权和销毁顺序设计。
3. 跑 `cargo fmt --all -- --check`。
4. 跑 `cargo clippy --all-targets --all-features -- -D warnings`。
5. 跑 `cargo test --all-targets`。
6. 对 Vulkan 运行任务，开启 validation layer 并记录是否有错误。
7. 更新对应文档中的状态、风险和下一步。
