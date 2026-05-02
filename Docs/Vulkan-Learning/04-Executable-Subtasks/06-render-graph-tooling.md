# 模块六可执行子任务：渲染图、调试工具、性能分析与工程化

导航：
- [第三层实现大任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/03-Implementation-Tasks/06-render-graph-tooling.md)
- [第五层开发计划](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/05-Development-Plans/06-render-graph-tooling.md)

## 子任务表

| 子任务 ID | 对应大任务 | 子任务内容 | 完成标志 |
| --- | --- | --- | --- |
| M6-S1 | M6-T1 | 封装 Vulkan object debug name | RenderDoc 对象可读 |
| M6-S2 | M6-T1 | 封装 command label scope | RenderDoc pass 可读 |
| M6-S3 | M6-T2 | 定义 render graph pass node | pass 可登记 |
| M6-S4 | M6-T2 | 定义资源读写声明 | graph 可知道依赖 |
| M6-S5 | M6-T2 | 编译 pass 执行顺序 | graph 可执行 |
| M6-S6 | M6-T3 | 实现 resource state tracker | 访问历史可记录 |
| M6-S7 | M6-T3 | 根据读写生成 barrier | 常见 pass 自动同步 |
| M6-S8 | M6-T4 | 建立 shader rebuild 命令 | shader 可增量重编 |
| M6-S9 | M6-T4 | 接入 pipeline cache | cache 可保存加载 |
| M6-S10 | M6-T5 | 创建 timestamp query pool | timestamp 可写入 |
| M6-S11 | M6-T5 | 输出 pass 级 GPU 时间 | 性能数据可读 |
| M6-S12 | M6-T6 | 建立 demo 启动约定 | demo 可回归 |
| M6-S13 | M6-T6 | 建立 benchmark scene | 性能基线可比较 |

## 执行提示

- M6-S1 到 M6-S2 越早做越好，调试收益很大。
- M6-S3 到 M6-S7 等多 pass 需求出现后再做。
- M6-S8 到 M6-S13 把项目从学习 demo 推向可长期维护的工程。
