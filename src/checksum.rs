use std::io;

// Checksum normalization is cross-platform. Hashing is also used by the
// remote attach download path on every target (Windows updates still run
// through install.ps1, but remote asset downloads are verified here too).
use std::{fs::File, io::Read, path::Path};

use sha2::{Digest, Sha256};

pub(crate) fn normalize_sha256(expected: &str) -> io::Result<String> {
    let expected = expected.trim().to_ascii_lowercase();
    if expected.len() != 64 || !expected.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected sha256 must be 64 hexadecimal characters",
        ));
    }
    Ok(expected)
}

pub(crate) fn verify_sha256(path: &Path, expected: &str) -> io::Result<()> {
    let expected = normalize_sha256(expected)?;
    let actual = file_sha256(path)?;
    if actual != expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("sha256 mismatch: expected {expected}, got {actual}"),
        ));
    }
    Ok(())
}

fn file_sha256(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(to_lower_hex(&hasher.finalize()))
}

fn to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn normalizes_valid_sha256_and_rejects_invalid_values() {
        assert_eq!(
            super::normalize_sha256(
                "  78193EF266C1E3C2CE4EA2A86D7FC87E8C52799653FAAAC8536533A1C9300F82  "
            )
            .unwrap(),
            "78193ef266c1e3c2ce4ea2a86d7fc87e8c52799653faaac8536533a1c9300f82"
        );
        assert!(super::normalize_sha256("").is_err());
        assert!(super::normalize_sha256("abc").is_err());
        assert!(super::normalize_sha256(&"g".repeat(64)).is_err());
    }

    #[test]
    fn verifies_matching_sha256() {
        use std::fs;

        let path = std::env::temp_dir().join(format!("herdr-checksum-test-{}", std::process::id()));
        fs::write(&path, b"herdr").unwrap();
        let result = super::verify_sha256(
            &path,
            "78193ef266c1e3c2ce4ea2a86d7fc87e8c52799653faaac8536533a1c9300f82",
        );
        let _ = fs::remove_file(&path);
        assert!(result.is_ok());
    }
}
