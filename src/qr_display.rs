//! QR code generation for plugin display.

use qrcode::render::unicode;
use qrcode::QrCode;

/// Generates QR code lines suitable for plugin rendering.
///
/// Returns individual lines that can be rendered at specific coordinates.
pub fn generate_qr_lines(data: &str) -> Result<Vec<String>, String> {
    if data.is_empty() {
        return Err("Cannot generate QR code from empty data".to_string());
    }

    let code = QrCode::new(data.as_bytes())
        .map_err(|e| format!("QR generation error: {}", e))?;

    let qr_string = code.render::<unicode::Dense1x2>().build();
    Ok(qr_string.lines().map(|s| s.to_string()).collect())
}
