# MiniEVM 工程计划

更新日期：2026-10-14。项目以 Rust 练手和理解执行引擎为目标，按小步迭代增加能力。
每轮先说明问题和行为，再实现、测试并同步文档。功能进入路线图不代表承诺交付日期。

## 技术基线

Rust 1.97.1、Edition 2024、Cargo、rustfmt、Clippy、rustdoc；无第三方 Cargo 依赖。
核心由自研 256 位 Word、20 字节 Address、栈、线性内存、BTreeMap 状态、简化 gas、
字节码解释器、CALL 和可选 tracer 组成。Git/GitHub 管理源码，Markdown 组织工程文档。

余额使用 u64，gas 与交易规则采用教学模型。后续功能按学习价值和测试能力选择，
不以立即构建完整 Ethereum 客户端为目标。

## 第一轮范围

| 工作 | 验收标准 | 状态 |
| --- | --- | --- |
| 零长度内存操作 | RETURN、REVERT、复制、日志、CALL 忽略不访问内存的偏移；RETURNDATACOPY 仍验证源范围 | 完成 |
| CALL 前置拒绝 | 深度、余额及 u64 溢出导致 CALL 返回 0，父执行继续，未开始的子调用不消耗转发 gas | 完成 |
| 测试断言 | 环境值、内存读回、截断 PUSH 验证实际值；增加边界与回滚回归 | 完成 |
| 错误诊断 | ExecutionResult 保留具体 VmError；提供 Display 和标准 Error 接口 | 完成 |
| 目录与代码 | 指令按分发、内存、调用、栈职责拆分；历史记录集中，公共入口清楚 | 完成 |
| 小步增强 | 增加 PUSH0、MSTORE8，验证 gas、边界及失败回滚 | 完成 |
| 文档与示例 | 更新架构、操作码、开发与测试索引；counter 展示 0 → 1 → 2；增加可编译 rustdoc 示例 | 完成 |

零长度与 CALL gas 是已复现的语义问题。接收账户余额溢出是本项目 u64 模型的边界，
本轮将它定义为 CALL 局部拒绝；真实 Ethereum 的余额表示不同。
新增指令适用于当前统一教学模型，不引入按 fork 启用或禁用指令的规则。

## 后续迭代候选

| 顺序 | 主题 | 开始前的条件与验收方向 |
| --- | --- | --- |
| 1 | 更多边界及数值测试 | 跨 limb 算术、移位、除零、栈边界；用可信实现或性质测试验证 Word |
| 2 | 自动质量门禁 | 已加入 GitHub Actions；后续继续维护 fmt、Clippy、测试、文档门禁 |
| 3 | 性能测量 | 先测算术循环、内存复制、storage、嵌套 CALL 和 tracer；记录工具链、输入、构建模式 |
| 4 | 状态与代码优化 | 基准证明成本后，用 journal 取代全量快照，缓存代码分析；验证多层回滚与缓存失效 |
| 5 | 追踪能力 | 有界记录已实现；后续评估采样、失败 PC/opcode、借用栈 |
| 6 | 指令元数据 | 名称、gas、最小栈深度、栈变化、立即数长度已统一；后续同步新增指令 |
| 7 | 执行功能 | 分别评估 KECCAK256、更多算术、STATICCALL/DELEGATECALL、CREATE、预编译 |
| 8 | 外围模块 | 根据示例需求考虑 ABI、Solidity 编译产物、交易编码与签名；再评估 RPC、CLI、持久化 |

前端、数据库、Docker、状态 trie、fork gas 和完整交易有效性属于后续按需选择的主题。
公共 API 的 rustdoc、测试公共夹具和测试分组也可随着代码增长持续整理。
安全报告入口需要维护者提供可用的私密联系方式，文档不得虚构联系地址或远程设置。

## 随机数与确定性

执行器对相同输入和初始状态必须产生相同结果。合约随机数通常是应用层问题。
未来 BLOCKHASH/PREVRANDAO 从环境读取确定的值；KECCAK256 对输入作确定的哈希。
VRF、commit–reveal 可作为后续合约示例。fuzzing 可使用随机输入，但须保存种子或失败样本。

## 验证与记录

每次修改执行语义需验证成功路径、边界、gas 和状态回滚。断言具体输出或状态，
不能仅以“执行成功”代表结果正确。性能建议需与实测收益区分。

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo doc --no-deps --locked
cargo run --locked --example counter
```

本轮新增 `ExecutionResult::error`、`VmError::NonceOverflow`，移除未使用的旧
`VmError::Reverted` 变体，属于 0.x 公共 API 调整；手工构造回执或穷举错误的下游代码需更新。

## 第一轮完成记录

基于提交 `5312118` 开始，2026-10-14 完成本地修改与验证。原有 33 个集成测试保留，
新增 17 个集成测试、1 个 PUSH 解码单元测试、1 个可编译文档测试；共 52 个测试通过。
环境指令和内存往返测试已改为断言真实值，PUSH 解码测试检查全部 32 种立即数宽度。

fmt、全目标 Clippy（警告视为错误）、完整测试、rustdoc 构建均通过。
检查 15 份文档中的 35 个本地链接，全部可解析；Git diff 空白检查通过。
两个示例运行成功：counter 为 0 → 1 → 2，每笔 160 gas；trace 返回 `2a`，使用 16 gas。

代码整理为 `instructions/{mod,memory,call,log,stack}.rs`，历史资料迁入 `docs/records/`，
新增 CONTRIBUTING、编辑器及换行约定。PUSH 使用固定数组，gas 表在编译期初始化；
交易校验移到快照前，避免发送账户及无效交易的额外复制。这些是明确的实现调整，
本轮没有进行基准测量，不声明具体速度收益。

第一轮结束时将 CI、状态 journal、代码缓存、追踪上限及统一指令元数据列为后续候选。

## 第二轮增强记录（2026-10-14）

在本轮之后又完成了 `MOD`、`GT` 和共享 `OpcodeInfo` 元数据表。名称、基础 gas、
最小栈深度、栈变化及 PUSH immediate 宽度现在由 `src/opcode/metadata.rs` 统一提供，
gas 计算和诊断名称不再分别维护。`LimitedTracer` 增加最大记录数及跳过快照的钩子，
在达到上限后跳过栈快照复制；新增固定种子数值测试覆盖 u128 对照、全宽进位、借位和除零。

GitHub Actions 位于 `.github/workflows/ci.yml`，固定 Rust 1.97.1，运行 fmt、Clippy、
测试和文档构建。此工作流是工程质量门禁，不等同于模糊测试、依赖审计或安全扫描。


第二轮验收：60 个集成测试、1 个单元测试、1 个文档测试，共 62 个测试全部通过。
fmt、全目标 Clippy（-D warnings）、文档构建、counter/trace 示例通过；38 个本地文档链接
可解析，diff 空白检查通过。MOD 和 DIV 共享一次长除法得到商与余数，减少重复运算。
元数据测试遍历 256 个 opcode，验证支持状态与空栈行为；有限追踪测试验证嵌套帧前缀、
执行结果不变以及跳过快照钩子。数字测试使用固定种子和原生 u128 作为独立对照。

本轮保持零第三方 Cargo 依赖。尚未测量性能收益，状态 journal、代码缓存和基准测试
仍是后续工作。GitHub 托管 CI 的结果须在推送后另行查看。
