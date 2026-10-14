//! Product image uploads: validation + storage.
//!
//! Files land in `UPLOAD_DIR` (default `data/uploads`, served at `/uploads/*`)
//! under random names. Validation is layered: extension, declared
//! content-type, magic bytes, and a 5MB cap.

use std::io::Write;

/// Max accepted upload size (5MB).
pub const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;

/// Allowed image types (extension + magic bytes + content-type).
pub const ALLOWED_TYPES: [(&str, &str); 4] = [
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("webp", "image/webp"),
];

fn magic_ok(ext: &str, bytes: &[u8]) -> bool {
    match ext {
        "png" => bytes.starts_with(&[0x89, b'P', b'N', b'G']),
        "jpg" | "jpeg" => bytes.starts_with(&[0xFF, 0xD8]),
        "webp" => bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        _ => false,
    }
}

/// Validate and store an uploaded image. Returns the public `/uploads/...`
/// URL path on success, or a short error code for redirects.
pub fn store_image(
    original_name: &str,
    content_type: &str,
    bytes: &[u8],
) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > MAX_UPLOAD_BYTES {
        return Err("size");
    }
    let ext = original_name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();
    let allowed = ALLOWED_TYPES.iter().find(|(e, _)| *e == ext);
    let Some((ext, mime)) = allowed else {
        return Err("filetype");
    };
    if !content_type.is_empty()
        && content_type != *mime
        && content_type != "application/octet-stream"
    {
        return Err("filetype");
    }
    if !magic_ok(ext, bytes) {
        return Err("filetype");
    }
    let dir = crate::config::upload_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("[upload] mkdir failed: {e}");
        return Err("io");
    }
    let name = format!(
        "{}-{}.{}",
        chrono::Utc::now().format("%Y%m%d%H%M%S"),
        &uuid::Uuid::new_v4().to_string()[..8],
        ext
    );
    let path = std::path::Path::new(&dir).join(&name);
    match std::fs::File::create(&path).and_then(|mut f| f.write_all(bytes)) {
        Ok(()) => Ok(format!("/uploads/{name}")),
        Err(e) => {
            eprintln!("[upload] write failed: {e}");
            Err("io")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_images() {
        assert!(store_image("x.exe", "application/x-msdownload", b"MZ...",).is_err());
        assert!(store_image("x.png", "image/png", b"not a png").is_err());
        assert!(store_image("x.png", "image/png", &[]).is_err());
        assert!(store_image("x.png", "image/png", &vec![0u8; MAX_UPLOAD_BYTES + 1]).is_err());
    }

    #[test]
    fn accepts_minimal_png() {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        bytes.extend_from_slice(&[0u8; 32]);
        let url = store_image("shot.PNG", "image/png", &bytes).unwrap();
        assert!(url.starts_with("/uploads/") && url.ends_with(".png"));
        // Cleanup the temp artifact.
        let path = std::path::Path::new(&crate::config::upload_dir())
            .join(url.trim_start_matches("/uploads/"));
        std::fs::remove_file(path).ok();
    }
}
