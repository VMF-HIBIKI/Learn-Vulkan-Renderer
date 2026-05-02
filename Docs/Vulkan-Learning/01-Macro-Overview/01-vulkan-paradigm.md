# Vulkan 范式与问题域

导航：
- [第一层索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/01-Macro-Overview/README.md)
- [学习路线图](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/01-Macro-Overview/04-learning-roadmap.md)

## Vulkan 到底在解决什么

Vulkan 是一套显式图形与计算 API。它把过去由驱动偷偷处理的大量事情交还给应用：

- 选择物理设备、队列族和扩展能力
- 创建逻辑设备、交换链、命令池和同步对象
- 显式分配 GPU 内存并绑定到 buffer / image
- 显式描述资源在 shader 中如何被访问
- 显式记录命令并提交到队列
- 显式声明资源状态转换、跨 pass 依赖和跨帧同步

因此学习 Vulkan 的关键不是记 API 名字，而是建立“CPU 生成 GPU 工作流”的模型。

## 与 OpenGL 式心智模型的区别

OpenGL 倾向于隐藏状态管理；Vulkan 倾向于暴露状态和成本。

- OpenGL 中一次 draw 可能隐含驱动校验、状态转换和资源调度。
- Vulkan 中 draw 只是已经录好的命令流中的一小段。
- Vulkan 的性能来自可预知性：对象提前创建，命令提前记录，资源状态由应用负责。

这也意味着错误更尖锐：同步漏写、布局错误、生命周期提前释放、描述符失效，都会直接表现为闪烁、黑屏、验证层报错或设备丢失。

## Rust 在这里扮演什么角色

Rust 不会让 Vulkan 自动安全。底层绑定仍然需要 `unsafe`，但 Rust 可以帮助我们把风险限制在清晰的边界内：

- 用 RAII 管理 Vulkan handle 的销毁顺序。
- 用类型封装区分 buffer、image、descriptor、pipeline 等资源。
- 用生命周期和所有权表达“资源由哪个上下文创建、在哪个队列使用、什么时候销毁”。
- 用小而稳定的 safe wrapper 把 `unsafe` 调用集中在后端模块。

本项目的原则是：`unsafe` 不扩散到渲染算法层；算法层应该面对明确的资源句柄和渲染图接口。

## 最终渲染器的问题域

最终目标不是一个 demo，而是一个具备以下能力的学习型渲染器：

- 可打开窗口并稳定呈现 swapchain 图像。
- 可上传 mesh、texture、material 和 camera 数据。
- 可执行深度、GBuffer、forward 或 deferred 光栅 pass。
- 可构建 BLAS / TLAS，并使用 ray tracing pipeline 或 ray query。
- 可把光追结果与光栅结果混合，例如阴影、反射、AO 或路径追踪 preview。
- 可通过 RenderDoc、验证层、日志、统计面板定位问题。

## 学习时最重要的约束

- 每个 Vulkan 对象都必须知道自己的创建者、所有者和销毁位置。
- 每个 GPU 资源都必须知道当前用途、布局和跨帧生命周期。
- 每个 pass 都必须声明输入、输出、读写方式和同步边。
- 每个高级效果都必须能退回到更小的可验证 demo。

## 第一版不做什么

- 不追求跨平台抽象层，先以 Windows + Vulkan SDK + desktop GPU 为主。
- 不从一开始写完整引擎，只做渲染器学习工程。
- 不封装成“看不见 Vulkan 的高级 API”，学习阶段要保留底层概念。
- 不急着做编辑器、资产热更新和复杂 ECS。
