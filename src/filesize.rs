/// Formats a byte size into human-readable decimal units (KB, MB, GB, TB, etc. with base 1000).
pub fn decimal(size: u64) -> String {
    const UNITS: [&str; 6] = ["bytes", "kB", "MB", "GB", "TB", "PB"];
    if size < 1000 {
        return format!("{size} bytes");
    }

    let mut val = size as f64;
    let mut unit_idx = 0;
    while val >= 1000.0 && unit_idx + 1 < UNITS.len() {
        val /= 1000.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", val, UNITS[unit_idx])
}

/// Formats a byte size into binary units (KiB, MiB, GiB, TiB, etc. with base 1024).
pub fn binary(size: u64) -> String {
    const UNITS: [&str; 6] = ["bytes", "KiB", "MiB", "GiB", "TiB", "PiB"];
    if size < 1024 {
        return format!("{size} bytes");
    }

    let mut val = size as f64;
    let mut unit_idx = 0;
    while val >= 1024.0 && unit_idx + 1 < UNITS.len() {
        val /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", val, UNITS[unit_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesize_decimal() {
        assert_eq!(decimal(500), "500 bytes");
        assert_eq!(decimal(1500), "1.5 kB");
        assert_eq!(decimal(2_500_000), "2.5 MB");
    }

    #[test]
    fn test_filesize_binary() {
        assert_eq!(binary(500), "500 bytes");
        assert_eq!(binary(1024), "1.0 KiB");
        assert_eq!(binary(1024 * 1024 * 5), "5.0 MiB");
    }
}
