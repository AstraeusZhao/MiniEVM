# Audit remediation record

Original audit: 2026-10-12. This record maps the original finding IDs to changes in the new maintenance baseline. Historical facts retain their provenance limitations; code changes do not retrospectively authenticate old dates or commit titles.

## Repository policy

The maintained branch uses the `Verify` CI check. Branch policy requires PRs, zero additional approvals for the single maintainer, resolved conversations and a current passing CI check, and rejects force pushes/deletion. Remote enforcement and exact new commit/CI IDs are verified after the audit push and recorded in the delivered cross-repository report. Security scan language scope is in `docs/SECURITY_SCAN_SCOPE.md`.

## GIT-001 — 六个默认分支均无保护规则或仓库 ruleset

Original evidence: GitHub branches API 对六个默认分支返回 protected=false；rulesets API 均返回空数组。分支保护 API 的 404 与这两个成功响应交叉印证。

Changes: Remote branch rules described above are enabled after the successful audit CI run.

Source references: workflow/configuration and history documents.

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## GIT-002 — 远端没有验证项目代码的自定义 CI 工作流

Original evidence: 六个默认分支均没有已跟踪的 .github/workflows 文件。DolphinPort Actions 中的动态 Dependabot Updates 不构成 Flutter/Rust/Android 构建验证；其余仓库 workflows 列表为空。DolphinPort 本地有被忽略的工作流，详见 DP-GIT-002。

Changes: 六仓库提交固定action SHA/Node/npm/pnpm/Go/Rust/Flutter/Foundry版本的可运行CI; CI包含构建、安全回归、官方依赖审计和Android APK原生payload检查

Source references: workflow/configuration and history documents.

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## GIT-003 — 安全告警 API 存在权限或功能缺口，不能以“无告警”判定安全

Original evidence: 六个仓库 Dependabot alerts 查询均失败，响应包含功能未启用和权限要求；多个 code-scanning/secret-scanning 查询返回 403/404。evm-patterns、MiniEVM 的 secret-scanning 查询成功且返回空列表，code-scanning 返回 no analysis found。逐接口状态在各项目远端表中完整列出。

Changes: 使用各仓库实际owner凭据消除安全告警读取权限缺口，启用Dependabot/secret scanning/push protection; 提交固定SHA CodeQL Actions/JS/Go/Rust扫描，明确Solidity/Dart不在CodeQL覆盖范围，区分404未配置与200无告警

Source references: workflow/configuration and history documents.

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## DEP-ENV-001 — 默认 npm 镜像不支持本次 audit 接口

Original evidence: 初次 npm audit 使用本机 registry.npmmirror.com，返回 404 Not Found / NOT_IMPLEMENTED。切换官方 registry.npmjs.org 后得到实际漏洞结果。该配置属于本机环境，不是六个项目均写入了此设置。

Changes: npm用户默认registry改官方npmjs，六仓库提交.npmrc并由CI显式审计; 移除冲突用户级GOROOT，默认go1.25.12与机器GOROOT一致；服务CI和本地验证显式使用Go1.26.5

Source references: workflow/configuration and history documents.

Validation: 默认go version/ go env GOROOT一致; npm config get registry=https://registry.npmjs.org/

## ENV-GO-001 — 默认 Go 可执行文件与 GOROOT 不匹配

Original evidence: 默认 go 是 1.25.12，但本机 GOROOT 指向 1.26.5，导致标准库 compile: version mismatch。改用 1.25.12 的目录后又出现标准库源码缺失。使用 C:\Dev\Go\1.26.5\bin\go.exe 并匹配 GOROOT 后，HerbTable API 和 UI captcha 后端 go vet 成功，HerbTable captcha 的未使用 import 被明确检出。

Changes: npm用户默认registry改官方npmjs，六仓库提交.npmrc并由CI显式审计; 移除冲突用户级GOROOT，默认go1.25.12与机器GOROOT一致；服务CI和本地验证显式使用Go1.26.5

Source references: workflow/configuration and history documents.

Validation: 默认go version/ go env GOROOT一致; npm config get registry=https://registry.npmjs.org/

## MINI-001 — Word 的派生 Ord 使用低位 limb 优先，数值比较和 DIV 错误

Original evidence: Word 的 [u64;4] 低位 limb 在 index0，但派生 Ord 从 index0 开始按数组字典序比较；unsigned_div 用 remainder>=rhs。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `src/primitives/word.rs`, `src/primitives/word.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## MINI-002 — CALL 按相反的参数顺序弹栈

Original evidence: 实际依次 pop out_size、out_offset、in_size、in_offset、value、to、gas；EVM CALL 的栈顶应首先取 gas，再取 to/value 等。源码注释也反写了栈顶。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `src/vm/instructions.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## MINI-003 — CALLDATACOPY/CODECOPY/RETURNDATACOPY 弹栈顺序颠倒

Original evidence: 复制指令先 pop size，随后 source/data offset，最后 memory offset；真实 EVM 首先 pop memory offset，最后才取 size。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `src/vm/instructions.rs`, `src/vm/instructions.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## MINI-004 — LOG 指令先弹 topics，真实 EVM 先取内存 offset/size

Original evidence: LOGn 在源码中先收集并反转 topics，再 pop size/offset；标准栈顶是 offset、size，随后依次 topics。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `src/vm/instructions.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## MINI-005 — 内存上限检查之前分配攻击者指定大小的 Vec

Original evidence: max_memory_bytes 默认 16 MiB，但 copy_padded 的 vec![0;size] 和 write_return_data 的 vec![0u8;size] 在 Memory.write 检查限额前分配。size 可由字节码控制，u64 范围并未限制到 max_memory_bytes。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `src/vm/calldata.rs`, `src/vm/instructions.rs`, `src/vm/instructions.rs`, `src/memory/linear.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## MINI-006 — REVERT 消耗全部 gas，没有保留剩余 gas

Original evidence: 顶层 Reverted 分支 gas_used=gas_limit；子调用 Reverted 分支 settle_child(...,0)。异常结构也没有携带执行到 REVERT 时的剩余 gas。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `src/vm/engine.rs`, `src/vm/instructions.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## MINI-007 — 执行失败后把 sender nonce 一并回滚

Original evidence: checkpoint 在 nonce++ 前建立；REVERT/Error 恢复整个 checkpoint。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `src/vm/engine.rs`, `src/vm/engine.rs`, `src/vm/engine.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## MINI-008 — BYTE 将大于 u64 的索引截断

Original evidence: BYTE 使用 index.low_u64()，没有检查更高 192 位；其他内存/跳转参数已经使用 low_u64_bounded，但 BYTE 未采用。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `src/vm/instructions.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## MINI-009 — 代码末尾不完整 PUSH 报错，标准 EVM 应补零

Original evidence: code.get(pc..end) 要求全部 immediate 字节存在，否则 CodeOutOfBounds。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `src/vm/instructions.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## MINI-010 — CALL 的返回区尾部被清零，标准只覆盖实际返回字节

Original evidence: write_return_data 建立 out_size 长度全零 buffer，再复制实际返回字节，写回整个区间。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `src/vm/instructions.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## MINI-011 — Gas 表和子调用转发属于简化模型，缺少硬分叉与资源收费

Original evidence: 多指令使用统一默认成本，SLOAD/SSTORE/LOG 等没有现代动态计费，缺内存二次项和 copy 按字计费；gas.forward 直接 min(request,remaining)，没有 EIP-150 的 63/64 上限。默认调用深度 64 与标准 1024 不同，余额域仅 u64。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `src/gas/schedule.rs`, `src/gas/meter.rs`, `src/vm/mod.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.

## MINI-012 — README 交易示例漏 nonce，tracing 接口未接入执行循环

Original evidence: README 的 Transaction struct literal 没有必填 nonce 字段；tracer/event 类型存在，但 Evm 不持有 tracer，execute 循环没有产生这些回调。README 提及 Clippy-gated CI，但远端无 workflow。

Changes: 数值Ord、栈顺序、分配前内存检查、REVERT剩余Gas和交易nonce、BYTE/PUSH/返回区修复; 新增真实字节码回归断言；更正旧测试错误约定; README明确教学Gas、u64余额和预留tracing范围，修正nonce示例

Source references: `README.md`, `src/core/transaction.rs`, `src/trace/tracer.rs`, `src/vm/engine.rs`

Validation: applicable local builds/regression checks completed before committing; the committed Audit CI reproduces the build and tests on GitHub. Exact results are in the cross-repository delivery report.
