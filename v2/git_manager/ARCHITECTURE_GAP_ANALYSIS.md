# V1 → V2 Architecture Gap Analysis

## Comparison Matrix

| Capability | V1 (Python) | V2 (Rust/Zig) | Gap | Priority | Architectural Impact |
|------------|-------------|----------------|-----|----------|---------------------|
| SSH key generation | ssh-keygen via subprocess | ZigSshProvider via FFI | None | ✅ | ✅ Properly abstracted |
| SSH config writing | ~/.ssh/gitmanager/config + Include | ~/.ssh/config directly (Zig FFI) | **MANAGED CONFIG + INCLUDE** | CRITICAL | Low — adapter-level change |
| SSH config backup | backup before edit | None | **BACKUP** | Medium | Low |
| Clone URL rewriting | HTTPS → SSH host alias URL | core.sshCommand with key path | **NONE NEEDED** (v2 is superior) | None | — |
| Clone fallback | SSH only | SSH → PAT → Password → Anonymous | V2 is superset | ✅ | ✅ |
| Clone tracking | None | DB-persisted CloneOperation | V2 is superset | ✅ | ✅ |
| Color schemes | 24+ via Rich themes | Single green banner | **THEME SYSTEM** | HIGH | Low — UI layer only |
| Theme persistence | theme.json | None | **PERSISTENCE** | HIGH | Low |
| Setup wizard | ssh setup-account (5-in-1) | 3 separate commands | **WIZARD** | Medium | Medium — new command |
| Multi-platform | 8 platforms | 8 platforms | None | ✅ | ✅ |
| SSH agent mgmt | ssh-agent via subprocess | Zig FFI | None | ✅ | ✅ |
| SSH connection test | ssh -T via subprocess | Zig FFI | None | ✅ | ✅ |
| Event system | None | Full event bus + audit | V2 is superset | ✅ | ✅ |
| CLI colors | Rich markup everywhere | owo-colors (hardcoded green) | **THEME-AWARE COLORS** | HIGH | Low |
| Error diagnostics | Rich Panels with solutions | Basic strings | **DIAGNOSTICS** | Medium | Low |
| Progress indicators | Rich Progress with spinners | indicatif | None functional | ✅ | ✅ |
| Interactive mode | Rich-based TUI | dialoguer-based TUI | None functional | ✅ | ✅ |
| REPL shell | None | Persistent subcommand shell | V2 is superset | ✅ | ✅ |
| DAG visualizer | None | Terminal DAG renderer | V2 is superset | ✅ | ✅ |

## Architecture Rules (Preserved)
✅ Hexagonal architecture — clean separation
✅ Ports and adapters — all infra behind traits
✅ Dependency inversion — domain pure
✅ Clone fallback engine — multi-strategy
✅ Event system — full audit trail
✅ Clone tracking — DB-persisted
✅ Plugin architecture — dynamic loading
✅ Multi-provider support — 8 platforms

## Architecture Rules (What NOT to do)
✗ Do NOT hardcode providers
✗ Do NOT bypass ports
✗ Do NOT introduce global state
✗ Do NOT duplicate clone logic
✗ Do NOT duplicate auth logic
✗ Do NOT break fallback engine
