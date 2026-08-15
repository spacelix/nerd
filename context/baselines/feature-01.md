# Feature 01 Baseline

## Measurement

- Date: 2026-08-15
- Build: `cargo build --locked --workspace --release`
- Application: `0.1.0-alpha.1`
- IPC protocol: `1`
- Rust: `1.97.0-x86_64-pc-windows-msvc`
- OS: Windows 11 Home Single Language x64, version `10.0.26200`
- CPU: 13th Gen Intel Core i5-13500H, 12 cores, 16 logical processors
- RAM: 23.7 GiB

`tests/windows/feature01-smoke.ps1 -Configuration release -ValidateElevatedRejection -AllowTemporaryLocalUser` first validates elevated-token rejection, then waits for daemon readiness, allows one second for startup work to settle, tests pipe access from a temporary second-user logon token, and samples process CPU for two seconds.

| Metric | Result | Feature budget | Status |
|---|---:|---:|---|
| Daemon working set | 10.6 MiB | less than 20 MiB | pass |
| Daemon private usage | 1.8 MiB | recorded, no separate MVP limit | recorded |
| Idle CPU over two seconds | 0.0 ms | near zero | pass |
| Graceful shutdown | 20.5 ms | less than 4 seconds | pass |
| `nerd-daemon.exe` | 2,792,960 bytes | installer budget measured later | recorded |
| `nerd.exe` | 616,448 bytes | installer budget measured later | recorded |
| `nerd-helper.exe` | 134,144 bytes | installer budget measured later | recorded |

## Acceptance Evidence

- `nerd status` returned daemon identity, protocol, uptime, health, data paths, and resource metrics.
- State, logging, IPC, and resource components reported healthy.
- Actual pipe DACL was protected and contained exactly two allow ACEs: LocalSystem and the active user SID. No Everyone, Anonymous, or Administrators ACE was present.
- A normal elevated daemon invocation exited with typed code `14` before creating `%LOCALAPPDATA%\Nerd`.
- Direct pipe access under a temporary second-user logon token failed with Windows `ERROR_ACCESS_DENIED`.
- A concurrent second daemon exited with typed already-running code `10`.
- `CTRL_BREAK` produced graceful exit code `0` in 20.5 ms, within the shared four-second shutdown deadline.
- After shutdown, `nerd status` returned daemon-absent code `3` and no daemon process remained.
- Test-created `%LOCALAPPDATA%\Nerd` state, temporary user, profile, and DPAPI-protected credential handoff were removed after handles closed.
- `cargo fmt --all -- --check`, strict workspace Clippy, all 28 workspace tests, release build, application-version consistency, and cargo-deny advisory/bans/license/source checks passed before the release measurement.
- Forced IPC deadline coverage verified that an active task is aborted, joined, and removed from the active-task registry.

## Scope

This is a developer baseline, not a Windows support certification. Windows 10 x64, Windows 11 release images, broader account configurations, and long-duration performance runs remain global release-matrix gates.
