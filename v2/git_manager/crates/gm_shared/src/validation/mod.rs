//! Input validation functions and the Git URL parser.
//!
//! This module sits at the bottom of the validation stack. Every piece of
//! user-supplied text that enters the system — aliases, emails, URLs — passes
//! through these functions before it reaches domain logic. Catching format
//! errors here means domain entities can trust that any string they receive
//! already has a valid shape; they only need to enforce business rules, not
//! format rules.

pub mod url_parser;
pub mod validators;

pub use validators::{ValidationError, validate_alias, validate_email, validate_url};
pub use url_parser::{GitUrl, GitProtocol, parse_git_url};