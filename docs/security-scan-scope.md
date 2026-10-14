# Security Scan Scope

MiniEVM has no third-party Cargo dependencies at this baseline. The tracked
[CI workflow](../.github/workflows/ci.yml) runs formatting, Clippy, tests and documentation
builds. This quality gate is not a security scanner. A successful build or empty alert
view does not establish the absence of vulnerabilities.

The workflow does not run fuzzing, dependency auditing, symbolic execution or a remote
security product. Remote security settings and hosted run results must be verified
separately. Record the exact commit and checks when reporting results.

Review should focus on untrusted bytecode, checked arithmetic and allocation, call and
transaction rollback, gas exhaustion, and memory limits. Limited tracing bounds stored
snapshots; per-frame memory and depth limits do not bound every aggregate resource.
