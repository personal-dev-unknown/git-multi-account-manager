# Git Manager — Architecture Overview

## System Philosophy

Git Manager is built on two complementary architectural patterns: **Microkernel Architecture** for the runtime and **Hexagonal Architecture (Ports & Adapters)** for the business logic boundaries. Together they produce a system where the core business logic — managing accounts, SSH keys, and git operations — is isolated from every infrastructure concern (databases, SSH binaries, platform APIs, UI frameworks). You can replace any infrastructure component without touching the domain.

## Layer Stack

The dependency arrows in this stack always point inward (downward in this list). A layer may only import from layers below it, never above.

```
Interfaces (CLI, Web, Desktop)
    ↓
Kernel (bootstrap, plugin loader, event bus, command bus)
    ↓
Ports (inbound commands, outbound trait definitions)
    ↓
Domain (pure business logic, domain entities, domain services)
    ↓
Shared (DTOs, errors, value objects, utilities)

Adapters → (implement Ports, link Zig, wrap SQLx) → everything above
Plugins → (implement Plugin + specific Port) → Kernel + Ports + Shared
```

### gm_shared

The foundation. Contains only serialisable data types — DTOs, error enumerations, and validation utilities. Nothing in this crate performs I/O, calls external APIs, or touches any runtime. Every other crate imports from here.

### gm_domain

Pure business logic. Entities enforce their invariants at construction time (an `Account` cannot be created with an invalid alias — the `new()` constructor returns `Err` if any rule is violated). Domain services orchestrate multi-step operations. Ports (trait definitions) declare what the domain needs from infrastructure without specifying how it works. This crate has zero runtime dependencies (no tokio, no sqlx, no reqwest).

### gm_ports

The hexagonal boundary. Declares two kinds of ports: *inbound* commands and queries (arriving from interface plugins) and *outbound* trait objects (called by the domain, implemented by adapters). This is where `RepositoryProvider`, `SshProvider`, `CredentialStore`, and `AuthProvider` are formally defined.

### gm_kernel

The microkernel runtime. Provides the service registry (TypeId → Arc mapping for dependency injection), the event bus (publish-subscribe for domain events), the command bus (middleware-wrapped command dispatch), the plugin lifecycle manager, the credential vault (AES-256-GCM encryption), and the workflow engine (multi-step coordinated operations). The kernel's `bootstrap()` function wires everything together at startup.

### gm_adapters

Bridges the Rust world to the outside world. Contains the Rust↔Zig FFI layer (for SSH key generation, SSH agent management, and git subprocess execution), SQLx repository implementations (MySQL for production, SQLite for development and tests), and the platform credential store (GNOME libsecret on Linux, Keychain on macOS).

### Provider Plugins

Each Git hosting platform (GitHub, GitLab, Bitbucket, Azure DevOps) is a plugin that implements two ports: `RepositoryProvider` (list and search remote repositories) and `AuthProvider` (validate credentials). Plugins register their concrete implementations in the kernel's service registry during `on_load()`. They never import from each other.

### Interface Plugins

The CLI, web, and desktop interfaces each implement `InterfacePlugin::run()`. They retrieve application services through the `CliServicesHandle` / `WebServicesHandle` facade registered by the binary entry point, then dispatch user requests to domain services. Interfaces never access the database directly.

## Key Design Decisions

**Why Zig for SSH and filesystem operations?** Zig provides direct OS-level access without the complexity of Rust's `unsafe` blocks scattered through business code. The Zig layer compiles to a static library (`libgm_native.a`) that Rust links at compile time. The C ABI boundary is the single point of contact between the two languages.

**Why a TypeId-keyed service registry instead of a DI framework?** Rust's type system enables zero-cost dependency injection without a framework. Plugins register concrete types at load time; consumers retrieve them by type. The registry uses `DashMap` for concurrent access without a global lock.

**Why CliServices / WebServices facade traits?** The interface plugins need to call domain services, but the domain services are generic (e.g., `AccountService<R: AccountRepository>`). The facade pattern erases these generic parameters: `apps/cli/main.rs` constructs `AccountService<MySqlAccountRepository>` and wraps it in a `ConcreteCliServices` that implements `CliServices`. The CLI command handlers only ever see `&dyn CliServices`.

## Credential Security

Credentials (PATs, OAuth tokens) flow through two encryption layers. The `CredentialVault` in the kernel encrypts with AES-256-GCM (using the `ring` crate) before any credential reaches the OS keychain. The OS keychain (GNOME libsecret, macOS Keychain) provides the second layer of encryption at rest. Neither layer stores keys in the database.