# 模块三开发计划：命令录制、同步模型与帧循环

导航：
- [第四层可执行子任务](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/04-Executable-Subtasks/03-command-sync-frame.md)
- [第二层设计文档](/e:/RustProject/Learn-Vulkan-Renderer/Docs/Vulkan-Learning/02-Module-Design/03-command-sync-frame.md)

## 推荐开发顺序

| 顺序 | 子任务 | 建议 Issue 标题 | 前置条件 | 测试重点 | 完成定义 |
| --- | --- | --- | --- | --- | --- |
| 1 | M3-S1 | `feat: create graphics command pool` | M1-S8 | queue family | pool 可创建 |
| 2 | M3-S2 | `feat: allocate primary command buffers` | M3-S1 | 分配/释放 | command buffer 可用 |
| 3 | M3-S3 | `feat: wrap command recording lifecycle` | M3-S2 | begin/end 状态 | recorder 可复用 |
| 4 | M3-S4 | `feat: define frame context` | M3-S3 | frame-local owner | 资源集中 |
| 5 | M3-S5 | `feat: create frame sync primitives` | M3-S4 | fence/semaphore | 同步对象可用 |
| 6 | M3-S6 | `feat: rotate frames in flight` | M3-S5 | fence wait/reset | 多帧不互踩 |
| 7 | M3-S7 | `feat: acquire swapchain images` | M1-S10, M3-S5 | image index | acquire 稳定 |
| 8 | M3-S8 | `feat: submit graphics work` | M3-S7 | wait/signal | submit 正确 |
| 9 | M3-S9 | `feat: present swapchain frames` | M3-S8 | out-of-date | present 稳定 |
| 10 | M3-S10 | `feat: transition image layouts` | M3-S3 | old/new layout | image barrier 可用 |
| 11 | M3-S11 | `feat: add explicit resource barriers` | M3-S10 | stage/access | barrier 可解释 |
| 12 | M3-S12 | `feat: add immediate submit helper` | M3-S5 | submit wait | 上传辅助可用 |
| 13 | M3-S13 | `feat: defer resource destruction by frame` | M3-S6 | fence 后释放 | 销毁安全 |

## 实施建议

- 每个 barrier 都写明前后访问，不要靠试错堆 flags。
- immediate submit 只用于离线或低频任务。
- 延迟销毁是 Vulkan 工程的安全网，尽早引入。

## 每个子任务的固定动作

1. 说明 CPU 等待点和 GPU 等待点。
2. 记录新增同步对象的生命周期。
3. 运行 `cargo fmt --all -- --check`。
4. 运行 `cargo clippy --all-targets --all-features -- -D warnings`。
5. 开启 validation layer，确认没有同步和 layout 错误。
