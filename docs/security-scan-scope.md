# Security Scan Scope

MiniEVM is a Rust project with no third-party Cargo dependencies at this baseline. The
repository does not currently define a tracked CI workflow or automated security scan.
An empty alert view or a successful local build is not evidence that the code has no
vulnerabilities.

Security review should focus on untrusted bytecode, checked arithmetic and allocation,
call and transaction rollback, gas exhaustion, and memory limits. Record the commit tested
and the exact checks run when reporting scan results.
