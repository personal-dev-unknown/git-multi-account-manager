use gm_domain::git::ports::CloneProgress;

/// Parses git clone --progress stderr output and extracts the progress percentage.
pub fn parse_progress_from_stderr(stderr: &str) -> Option<CloneProgress> {
    let mut last_progress: Option<CloneProgress> = None;

    for line in stderr.lines() {
        if let Some(pct) = parse_percentage(line) {
            let mut progress = CloneProgress::new(pct);
            progress.message = line.to_string();

            if let Some((received, total)) = parse_byte_counts(line) {
                progress.bytes_transferred = received;
                progress.total_bytes = total;
            }
            if let Some(speed) = parse_speed(line) {
                progress.speed_bps = speed;
            }
            last_progress = Some(progress);
        }
    }

    last_progress
}

/// Extracts a percentage value from a git progress line.
/// Handles "Receiving objects:  45% (1234/2743)" and "Resolving deltas:  67% (456/678)".
fn parse_percentage(line: &str) -> Option<f64> {
    // Look for pattern: number followed by %
    let bytes = line.as_bytes();
    let len = bytes.len();
    if len < 2 {
        return None;
    }

    // Find the '%' character and work backwards
    let pct_pos = line.find('%')?;
    if pct_pos == 0 {
        return None;
    }

    // Find the start of the number before '%'
    let num_start = line[..pct_pos]
        .rfind(|c: char| !c.is_ascii_digit() && c != '.')
        .map(|p| p + 1)
        .unwrap_or(0);

    let num_str = &line[num_start..pct_pos].trim();
    num_str.parse::<f64>().ok()
}

/// Parses byte counts from "(received/total)" in a progress line.
fn parse_byte_counts(line: &str) -> Option<(u64, u64)> {
    // Match pattern like "(1234/2743)"
    let open = line.find('(')?;
    let close = line[open..].find(')')?;
    let inner = &line[open + 1..open + close];

    let mut parts = inner.splitn(2, '/');
    let received = parts.next()?.trim().parse::<u64>().ok()?;
    let total = parts.next()?.trim().parse::<u64>().ok()?;
    Some((received, total))
}

/// Parses speed from lines like "1.2 MiB | 1.5 MiB/s"
fn parse_speed(line: &str) -> Option<u64> {
    // Find the pipe separator and get the part after it
    let pipe = line.rfind('|')?;
    let speed_part = line[pipe + 1..].trim();

    // Match patterns like "1.5 MiB/s", "500 KiB/s", "2.3 GiB/s"
    let mut parts = speed_part.splitn(2, ' ');
    let value_str = parts.next()?.trim();
    let unit_str = parts.next()?.trim().to_lowercase();

    let value: f64 = value_str.parse().ok()?;
    let multiplier = if unit_str.starts_with("kib") {
        1024.0
    } else if unit_str.starts_with("mib") {
        1024.0 * 1024.0
    } else if unit_str.starts_with("gib") {
        1024.0 * 1024.0 * 1024.0
    } else {
        return None;
    };

    Some((value * multiplier) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_receiving_objects() {
        let line = "Receiving objects:  45% (1234/2743), 1.2 MiB | 1.5 MiB/s";
        assert_eq!(parse_percentage(line), Some(45.0));
        assert_eq!(parse_byte_counts(line), Some((1234, 2743)));
        // The pipe rfind matches "| 1.5 MiB/s" (speed after the pipe)
        assert_eq!(parse_speed(line), Some((1.5 * 1024.0 * 1024.0) as u64));
    }

    #[test]
    fn parses_resolving_deltas() {
        let line = "Resolving deltas:  67% (456/678)";
        assert_eq!(parse_percentage(line), Some(67.0));
        assert_eq!(parse_byte_counts(line), Some((456, 678)));
        assert!(parse_speed(line).is_none());
    }

    #[test]
    fn parses_complete_progress() {
        let stderr = "Cloning into '/tmp/repo'...
Receiving objects:  45% (1234/2743), 1.2 MiB | 1.5 MiB/s
Receiving objects:  100% (2743/2743), 2.1 MiB | 1.5 MiB/s
Resolving deltas:  100% (678/678)";
        let progress = parse_progress_from_stderr(stderr);
        assert!(progress.is_some());
        let p = progress.unwrap();
        assert!((p.percentage - 100.0).abs() < 0.01);
        // The last progress line has (678/678) objects
        assert_eq!(p.bytes_transferred, 678);
    }

    #[test]
    fn handles_empty_stderr() {
        assert!(parse_progress_from_stderr("").is_none());
    }

    #[test]
    fn handles_no_progress_lines() {
        let stderr = "fatal: repository not found";
        assert!(parse_progress_from_stderr(stderr).is_none());
    }
}
