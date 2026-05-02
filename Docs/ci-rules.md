# CI 与协作规则

本文档定义本仓库的 CI 要求和协作约束。下文中的 Merge Request 在 GitHub 中对应 Pull Request（PR）。

## 目标

- `master` 是唯一主分支。
- 任何改动都不能直接提交到 `master`。
- 任何 PR 都必须先从一个 Issue 开始创建。
- Issue、Commit、文档计划和 CI 结果需要形成完整的可追溯链路。

## 分支与合并策略

1. `master` 仅接收通过 CI 的 PR。
2. 不允许绕过 PR 直接向 `master` 推送代码。
3. 新需求、修复、重构、文档更新、实验 demo 都必须先创建 Issue。
4. 创建 Issue 后，从该 Issue 派生工作分支，再提交 PR 到 `master`。

## Issue 规范

Issue 标题必须符合以下格式：

```text
<type>: <short summary>
```

允许的 `type` 只有：

```text
build | ci | docs | feat | fix | perf | refactor | test | demo | bench
```

示例：

```text
docs: describe vulkan device bootstrap plan
feat: create vulkan instance and debug messenger
demo: render first swapchain triangle
bench: measure staging upload bandwidth
```

每个 Issue 都必须至少分配给 1 个 assignee。

## Commit 规范

Commit 标题必须符合以下格式：

```text
<type>: <short summary> (#<issue ID>)
```

示例：

```text
docs: add renderer learning roadmap (#12)
feat: initialize vulkan instance (#13)
```

补充说明：

- Commit 必须关联一个 Issue，且该 Issue 必须已经分配给至少 1 个 assignee。
- 每个 Issue 只能对应 1 个非 merge commit。
- 如果需要修复已存在的 commit，使用 `git commit --amend` 后再 force push 到工作分支。
- Commit 尾部的 `#<issue ID>` 必须与工作分支名前缀中的 issue ID 一致。

## PR 规范

1. PR 的目标分支必须是 `master`。
2. PR 对应的工作分支应从 Issue 创建，建议分支名以 Issue 编号开头，例如 `12-vulkan-bootstrap`。
3. 每个 PR 只能包含 1 个非 merge commit。
4. 若提交后发现问题，必须在原 commit 上 `amend`。
5. PR 合并前，必须通过仓库 CI。
6. PR 必须至少获得 1 次审批后才能合并。
7. 每个 PR 都必须至少分配给 1 个 assignee。

## PR 创建前检查清单

1. 已创建 Issue，且 Issue 标题符合规范。
2. Issue 至少有 1 个 assignee。
3. 工作分支名以 Issue ID 开头，例如 `12-vulkan-bootstrap`。
4. Commit 标题以 `(#<issue ID>)` 结尾，且 Issue ID 与分支前缀一致。
5. PR 创建后立刻分配至少 1 个 assignee。
6. 若任务完成了一个可执行子任务，已更新 `Docs/Knowledge-Graph` 中对应知识图谱。

## CI 检查内容

仓库 CI 计划执行以下检查：

1. Issue 标题格式检查。
2. Issue assignee 检查。
3. PR 分支命名检查。
4. PR assignee 检查。
5. 单 Issue 单 Commit 检查。
6. Commit 信息与分支 issue ID 一致性检查。
7. Commit 关联 Issue 的 assignee 检查。
8. `cargo fmt --all -- --check`
9. `cargo clippy --all-targets --all-features -- -D warnings`
10. `cargo test --all-targets`

## Vulkan 额外约束

- 所有 `unsafe` Vulkan 调用都必须在封装层说明安全不变量。
- 资源创建、销毁、队列提交和同步相关改动必须有 smoke test、验证层运行记录或 RenderDoc 截图记录之一。
- 新增 GPU 资源类型时，必须在文档中说明所有权、销毁顺序、内存绑定方式和跨帧生命周期。
- 新增渲染 pass 时，必须说明输入资源、输出资源、布局转换和同步边。
