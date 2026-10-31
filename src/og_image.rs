//! Pixel size and MIME type for a PNG, JPEG, or WebP share image.
//! Headers only: the file body is not decoded.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RasterSize {
    pub width: u32,
    pub height: u32,
    pub mime: &'static str,
}

/// `assets/images/…` path relative to `source/images/`, with traversal rejected.
pub fn source_image_rel(url: &str) -> Option<&str> {
    let v = url.trim();
    let v = v.split(['?', '#']).next().unwrap_or(v).trim();
    let v = v.trim_start_matches('/');
    let rest = v.strip_prefix("assets/images/")?;
    if rest.is_empty()
        || rest
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    Some(rest)
}

pub fn mime_from_url(url: &str) -> Option<&'static str> {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let segment = path.rsplit('/').next().unwrap_or(path);
    let (name, ext) = segment.rsplit_once('.')?;
    if name.is_empty() {
        return None;
    }
    match ext.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

pub fn raster_size(bytes: &[u8]) -> Option<RasterSize> {
    png_size(bytes)
        .or_else(|| jpeg_size(bytes))
        .or_else(|| webp_size(bytes))
}

fn png_size(bytes: &[u8]) -> Option<RasterSize> {
    const SIG: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.len() < 24 || &bytes[..8] != SIG || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    positive(width, height, "image/png")
}

fn jpeg_size(bytes: &[u8]) -> Option<RasterSize> {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return None;
    }
    let mut i = 2;
    while i + 1 < bytes.len() {
        if bytes[i] != 0xFF {
            return None;
        }
        while i < bytes.len() && bytes[i] == 0xFF {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        let marker = bytes[i];
        i += 1;
        // Start of scan and end of image have no size we can use.
        if marker == 0xDA || marker == 0xD9 {
            return None;
        }
        // Standalone markers (TEM and restart) carry no length.
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            continue;
        }
        if i + 1 >= bytes.len() {
            return None;
        }
        let len = u16::from_be_bytes([bytes[i], bytes[i + 1]]) as usize;
        if len < 2 || i + len > bytes.len() {
            return None;
        }
        if is_sof(marker) {
            if len < 7 {
                return None;
            }
            let height = u16::from_be_bytes([bytes[i + 3], bytes[i + 4]]) as u32;
            let width = u16::from_be_bytes([bytes[i + 5], bytes[i + 6]]) as u32;
            return positive(width, height, "image/jpeg");
        }
        i += len;
    }
    None
}

fn is_sof(marker: u8) -> bool {
    matches!(
        marker,
        0xC0 | 0xC1 | 0xC2 | 0xC3 | 0xC5 | 0xC6 | 0xC7 | 0xC9 | 0xCA | 0xCB | 0xCD | 0xCE | 0xCF
    )
}

fn webp_size(bytes: &[u8]) -> Option<RasterSize> {
    if bytes.len() < 16 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }
    let mut i = 12;
    while i + 8 <= bytes.len() {
        let tag = &bytes[i..i + 4];
        let size = u32::from_le_bytes(bytes[i + 4..i + 8].try_into().ok()?) as usize;
        let data = i + 8;
        if tag == b"VP8X" && data + 10 <= bytes.len() {
            let width =
                1 + u32::from_le_bytes([bytes[data + 4], bytes[data + 5], bytes[data + 6], 0]);
            let height =
                1 + u32::from_le_bytes([bytes[data + 7], bytes[data + 8], bytes[data + 9], 0]);
            return positive(width, height, "image/webp");
        }
        if tag == b"VP8 "
            && data + 10 <= bytes.len()
            && bytes.get(data + 3..data + 6) == Some(&[0x9D, 0x01, 0x2A])
        {
            let width = u16::from_le_bytes([bytes[data + 6], bytes[data + 7]]) as u32 & 0x3FFF;
            let height = u16::from_le_bytes([bytes[data + 8], bytes[data + 9]]) as u32 & 0x3FFF;
            return positive(width, height, "image/webp");
        }
        if tag == b"VP8L" && data + 5 <= bytes.len() && bytes[data] == 0x2F {
            let b1 = bytes[data + 1] as u32;
            let b2 = bytes[data + 2] as u32;
            let b3 = bytes[data + 3] as u32;
            let b4 = bytes[data + 4] as u32;
            let width = 1 + (b1 | ((b2 & 0x3F) << 8));
            let height = 1 + (((b2 & 0xC0) >> 6) | (b3 << 2) | ((b4 & 0x0F) << 10));
            return positive(width, height, "image/webp");
        }
        let padded = size + (size & 1);
        if data + padded < data {
            return None;
        }
        i = data + padded;
    }
    None
}

fn positive(width: u32, height: u32, mime: &'static str) -> Option<RasterSize> {
    if width == 0 || height == 0 {
        None
    } else {
        Some(RasterSize {
            width,
            height,
            mime,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_ihdr_width_and_height() {
        let mut bytes = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, b'I', b'H',
            b'D', b'R',
        ];
        bytes.extend(640u32.to_be_bytes());
        bytes.extend(480u32.to_be_bytes());
        let size = raster_size(&bytes).expect("png");
        assert_eq!(size.width, 640);
        assert_eq!(size.height, 480);
        assert_eq!(size.mime, "image/png");
    }

    #[test]
    fn jpeg_sof_width_and_height() {
        let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x0B, 0x08];
        bytes.extend(418u16.to_be_bytes());
        bytes.extend(800u16.to_be_bytes());
        // Length includes these component bytes so the segment stays inside the buffer.
        bytes.extend([1, 1, 1, 0]);
        let size = raster_size(&bytes).expect("jpeg");
        assert_eq!(size.width, 800);
        assert_eq!(size.height, 418);
        assert_eq!(size.mime, "image/jpeg");
    }

    #[test]
    fn webp_vp8x_width_and_height() {
        let mut bytes = b"RIFF\0\0\0\0WEBPVP8X".to_vec();
        bytes.extend(10u32.to_le_bytes());
        bytes.extend([0, 0, 0, 0]);
        let width = 1199u32.to_le_bytes();
        let height = 629u32.to_le_bytes();
        bytes.extend_from_slice(&width[..3]);
        bytes.extend_from_slice(&height[..3]);
        let size = raster_size(&bytes).expect("webp");
        assert_eq!(size.width, 1200);
        assert_eq!(size.height, 630);
        assert_eq!(size.mime, "image/webp");
    }

    #[test]
    fn mime_and_local_path() {
        assert_eq!(
            mime_from_url("https://ex.com/a/photo.JPG?x=1"),
            Some("image/jpeg")
        );
        assert_eq!(
            mime_from_url("/assets/images/og-default.png"),
            Some("image/png")
        );
        assert_eq!(
            source_image_rel("/assets/images/og/default.webp"),
            Some("og/default.webp")
        );
        assert!(source_image_rel("/assets/images/../secret.png").is_none());
        assert!(source_image_rel("https://ex.com/a.jpg").is_none());
    }
}
