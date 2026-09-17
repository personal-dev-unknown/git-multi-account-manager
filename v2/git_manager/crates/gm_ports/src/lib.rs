//! gm_ports — the hexagonal boundary layer.
//!
//! This crate formalises two directions of communication:
//!
//! **Inbound (left side of the hexagon)** — commands and queries that arrive
//! from interface plugins (CLI, Web, Desktop). Each command is a plain Rust
//! struct carrying the data the user supplied. The kernel's command bus receives
//! these, passes them through middleware, and routes them to domain service
//! handlers.
//!
//! **Outbound (right side of the hexagon)** — traits that the domain and kernel
//! call when they need infrastructure: a git executor, an SSH provider, a
//! credential store. Each trait is implemented by an adapter in `gm_adapters`.
//! The kernel wires the concrete adapter types into the service registry at boot.
//!
//! Neither the inbound commands nor the outbound traits carry any logic —
//! they are pure contracts. Business logic lives in `gm_domain`, and the
//! concrete I/O implementations live in `gm_adapters`.

pub mod inbound;
pub mod outbound;