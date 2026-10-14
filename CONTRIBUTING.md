# Contributing

MiniEVM grows through small, tested additions to its execution engine. Read the
[engineering plan](docs/engineering-plan.md), [architecture](docs/architecture.md), and
[development guide](docs/development.md) before extending it.

1. Describe the behavior, its scope, and expected output or state transition.
2. Add a regression or feature test that checks those observable results.
3. Update code, opcode metadata and relevant documentation together.
4. Run the checks in the development guide and report their results.

Use the pinned toolchain and keep changes focused. Explain public API changes and new
dependencies. Suggested future work belongs in the plan until it has implementation and
verification. Follow the [Code of Conduct](docs/code-of-conduct.md); execution integrity
and resource-use reports should follow the [Security Policy](docs/security.md).
