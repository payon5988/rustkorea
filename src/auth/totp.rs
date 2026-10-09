use totp_rs::{Algorithm, Secret, TOTP};

/// provision a fresh secret (base32) for display + storage.
pub fn new_secret() -> String {
    Secret::generate_secret().to_encoded().to_string()
}

fn totp(secret_b32: &str, account: &str) -> Option<TOTP> {
    let secret = Secret::Encoded(secret_b32.to_uppercase());
    let bytes = secret.to_bytes().ok()?;
    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        bytes,
        Some("RustKorea".to_string()),
        account.to_string(),
    )
    .ok()
}

/// QR code (base64 PNG data URI) for camera registration. Returns `None`
/// when the otpauth URL is too long to encode.
pub fn qr_data_uri(secret_b32: &str, account: &str) -> Option<String> {
    let t = totp(&secret_b32.to_uppercase(), account)?;
    let png_b64 = t.get_qr_base64().ok()?;
    Some(format!("data:image/png;base64,{png_b64}"))
}

/// otpauth:// URI for manual entry into authenticator apps.
pub fn provision_uri(secret_b32: &str, account: &str) -> Option<String> {
    use std::fmt::Write;
    let secret = secret_b32.to_uppercase();
    // Validate by constructing the verifier first.
    totp(&secret, account)?;
    let mut label = String::new();
    for c in account.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~' | '@') {
            label.push(c);
        } else {
            write!(label, "%{:02X}", c as u32).ok()?;
        }
    }
    Some(format!(
        "otpauth://totp/RustKorea:{label}?secret={secret}&issuer=RustKorea&algorithm=SHA1&digits=6&period=30"
    ))
}

/// Verify a 6-digit code with ±1 step skew.
pub fn verify(secret_b32: &str, account: &str, code: &str) -> bool {
    let code: String = code.chars().filter(|c| c.is_ascii_digit()).collect();
    if code.len() != 6 {
        return false;
    }
    totp(secret_b32, account)
        .and_then(|t| t.check_current(&code).ok())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let secret = new_secret();
        assert!(!secret.is_empty());
        let t = totp(&secret, "admin@example.com").unwrap();
        let code = t.generate_current().unwrap();
        assert!(verify(&secret, "admin@example.com", &code));
        assert!(!verify(&secret, "admin@example.com", "000000"));
        assert!(!verify(&secret, "admin@example.com", "12"));
    }

    #[test]
    fn provision_uri_shape() {
        let uri = provision_uri("JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP", "admin@example.com").unwrap();
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("secret=JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP"));
    }

    #[test]
    fn qr_is_png_data_uri() {
        let uri = qr_data_uri("JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP", "admin@example.com").unwrap();
        assert!(uri.starts_with("data:image/png;base64,iVBORw0KGgo"));
    }
}
