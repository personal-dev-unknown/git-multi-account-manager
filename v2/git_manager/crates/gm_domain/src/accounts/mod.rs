//! The accounts bounded context.
//!
//! An account represents a single Git hosting account (e.g., one GitHub user).
//! A person may have many accounts across different platforms or even multiple
//! accounts on the same platform (a personal account and a work account on GitHub).
//!
//! Every other domain concept — repositories, SSH keys, sync sessions — belongs
//! to an account. The account is the root aggregate of this bounded context.

pub mod entities;
pub mod events;
pub mod ports;
pub mod services;
pub mod value_objects;