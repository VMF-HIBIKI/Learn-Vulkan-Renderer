# 模块六实现大任务：渲染图、调试工具、性能分析与工程化

导航：
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/06-render-graph-tooling.md)
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/06-render-graph-tooling.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/06-render-graph-tooling.md)

## 大任务总览

| 任务 ID | 名称 | 目标 |
| --- | --- | --- |
| M6-T1 | Debug Name 与 Label | 让 RenderDoc 捕获中的对象和 pass 可读 |
| M6-T2 | Render Graph 初版 | 声明 pass 读写资源并编译执行顺序 |
| M6-T3 | Barrier 自动化 | 根据资源访问历史生成基础 barrier |
| M6-T4 | Shader 与 Pipeline 工程化 | 支持 shader rebuild、pipeline cache 和错误报告 |
| M6-T5 | GPU Profiling | 使用 timestamp query 统计 pass 级耗时 |
| M6-T6 | 回归 Demo 与基准 | 建立可持续验证渲染能力的 demo 集 |

## M6-T1 Debug Name 与 Label

产出：

- Vulkan object naming helper
- command buffer debug label
- pass scope label

验收标准：

- RenderDoc 中对象名称可读
- 每个 pass 有清晰 marker
- debug utils 不可用时能降级

## M6-T2 Render Graph 初版

产出：

- pass node
- resource read/write declaration
- graph compile
- execute callback

验收标准：

- clear、depth、mesh、post process 可通过 graph 排序执行
- pass 缺少资源声明时能报错
- graph 输出可打印用于调试

## M6-T3 Barrier 自动化

产出：

- resource state tracker
- read/write access 推导
- image layout transition 推导
- graph barrier emission

验收标准：

- 常见 color/depth/sampled/storage 转换自动生成
- 生成 barrier 可被日志解释
- 验证层无同步和 layout 错误

## M6-T4 Shader 与 Pipeline 工程化

产出：

- shader 编译命令
- shader dependency tracking
- pipeline cache
- pipeline error report

验收标准：

- shader 修改后可重建
- pipeline cache 可保存和加载
- 编译失败不导致无信息崩溃

## M6-T5 GPU Profiling

产出：

- timestamp query pool
- pass begin/end timestamp
- frame timing report

验收标准：

- 能输出每个 pass 的 GPU 时间
- query 读取等待逻辑正确
- 性能数据与 RenderDoc 大致可交叉验证

## M6-T6 回归 Demo 与基准

产出：

- triangle demo
- textured mesh demo
- PBR scene demo
- ray traced shadow demo
- benchmark scene

验收标准：

- 每个 demo 有明确启动方式和预期画面
- 改动后能快速确认核心能力未退化
- 性能基线可记录和比较

## 风险与边界

- Render graph 不要比实际渲染需求复杂太多。
- Profiling 数据要等 GPU 完成后读，不能破坏帧循环。
- 工具链应帮助定位问题，而不是掩盖底层 Vulkan 语义。
