/// Simple form-urlencoding compatible with query strings and path segments.
/// Unreserved characters (A-Z, a-z, 0-9, -, _, ., ~) pass through unchanged.
/// Spaces become `+` (form-urlencoding convention). All other bytes are
/// percent-encoded as `%XX`.
pub fn encode(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => '+'.to_string(),
        other => format!("%{:02X}", other as u32),
    }).collect()
}
