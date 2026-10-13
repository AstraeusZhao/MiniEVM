# Dependency Baseline

MiniEVM currently has no third-party Cargo dependencies. The Rust toolchain is pinned to
1.97.1 in `rust-toolchain.toml`, and `Cargo.lock` is committed for reproducible builds.

When adding a dependency, document why it is needed, review its license and maintenance
status, and update the lockfile with the change. Dependency review does not replace code
review or execution tests.
