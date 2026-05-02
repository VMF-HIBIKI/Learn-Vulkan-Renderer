# 第一层：宏观理解与术语地图

返回根索引：[Vulkan Learning](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/README.md)

## 阅读顺序

1. [Vulkan 范式与问题域](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/01-Macro-Overview/01-vulkan-paradigm.md)
2. [渲染器运行时架构全景](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/01-Macro-Overview/02-runtime-architecture.md)
3. [Vulkan 参考映射](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/01-Macro-Overview/03-vulkan-reference-map.md)
4. [学习与实现路线图](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/01-Macro-Overview/04-learning-roadmap.md)

## 本层目标

- 先把 Vulkan 看成一套显式 GPU 工作提交系统，而不是“更复杂的 OpenGL”。
- 确认渲染器中哪些能力是底层平台能力，哪些是渲染算法与工程组织。
- 确认我们应该先实现可验证的最小闭环，再逐步进入光栅、PBR、光追和渲染图。

## 下钻入口

- 第二层入口：[模块设计索引](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/README.md)
