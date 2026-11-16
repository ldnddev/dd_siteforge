//! Copy a picked or dropped file into `source/images/`.
//!
//! Decision 10: copy as-is by default. When the caller asks, downscale to
//! 2560 px on the long edge if the image is larger. Never upscale. Keep the
//! format. Kebab-case the file name. Never overwrite: the first clash is `-2`.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::{GenericImageView, ImageFormat};

pub const LONG_EDGE: u32 = 2560;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedImage {
    pub filename: String,
    /// Value stored on the field. The TUI picker writes `assets/images/{name}`.
    pub stored: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// True when the bytes written are the bytes that were read.
    pub copied_as_is: bool,
}

/// Long edge in pixels, when the file decodes as a raster image.
pub fn long_edge(bytes: &[u8]) -> Option<u32> {
    let image = image::load_from_memory(bytes).ok()?;
    let (width, height) = image.dimensions();
    Some(width.max(height))
}

pub fn kebab_filename(original: &str) -> String {
    let path = Path::new(original);
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("image");
    let mut safe = String::new();
    let mut dash = false;
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() {
            safe.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !dash && !safe.is_empty() {
            safe.push('-');
            dash = true;
        }
    }
    while safe.ends_with('-') {
        safe.pop();
    }
    if safe.is_empty() {
        safe = "image".to_string();
    }
    if ext.is_empty() {
        safe
    } else {
        format!("{safe}.{ext}")
    }
}

/// `name`, or `name-2`, `name-3`, … when `name` is already in `dir`.
pub fn clash_name(dir: &Path, filename: &str) -> String {
    if !dir.join(filename).exists() {
        return filename.to_string();
    }
    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("image");
    let ext = path.extension().and_then(|ext| ext.to_str());
    let mut n = 2u32;
    loop {
        let candidate = match ext {
            Some(ext) => format!("{stem}-{n}.{ext}"),
            None => format!("{stem}-{n}"),
        };
        if !dir.join(&candidate).exists() {
            return candidate;
        }
        n += 1;
    }
}

pub fn stored_value(filename: &str) -> String {
    format!("assets/images/{filename}")
}

/// Copy `bytes` into `images_dir` (`source/images`).
///
/// `downscale` is the optional shrink. GIF, SVG, and anything that is not a
/// still PNG, JPEG, or WebP stay byte-for-byte: shrinking them would drop
/// frames or change the format.
pub fn import_bytes(
    original_name: &str,
    bytes: &[u8],
    images_dir: &Path,
    downscale: bool,
) -> anyhow::Result<ImportedImage> {
    std::fs::create_dir_all(images_dir)?;
    let filename = clash_name(images_dir, &kebab_filename(original_name));
    let dest = images_dir.join(&filename);
    let ext = Path::new(&filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let (payload, width, height, copied_as_is) = encode(ext.as_str(), bytes, downscale)?;
    std::fs::write(&dest, &payload)?;
    Ok(ImportedImage {
        stored: stored_value(&filename),
        filename,
        width,
        height,
        copied_as_is,
    })
}

pub fn import_path(
    src: &Path,
    images_dir: &Path,
    downscale: bool,
) -> anyhow::Result<ImportedImage> {
    let bytes = std::fs::read(src)?;
    let name = src
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("image");
    import_bytes(name, &bytes, images_dir, downscale)
}

fn encode(
    ext: &str,
    bytes: &[u8],
    downscale: bool,
) -> anyhow::Result<(Vec<u8>, Option<u32>, Option<u32>, bool)> {
    if matches!(ext, "svg" | "gif" | "mp4") || !downscale {
        let edge = long_edge(bytes);
        return Ok((bytes.to_vec(), edge, None, true));
    }
    let format = match ext {
        "png" => ImageFormat::Png,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "webp" => ImageFormat::WebP,
        _ => {
            let edge = long_edge(bytes);
            return Ok((bytes.to_vec(), edge, None, true));
        }
    };
    let image = match image::load_from_memory(bytes) {
        Ok(image) => image,
        Err(_) => return Ok((bytes.to_vec(), None, None, true)),
    };
    let (width, height) = image.dimensions();
    let long = width.max(height);
    if long <= LONG_EDGE {
        return Ok((bytes.to_vec(), Some(width), Some(height), true));
    }
    let scale = LONG_EDGE as f32 / long as f32;
    let next_w = ((width as f32) * scale).round().max(1.0) as u32;
    let next_h = ((height as f32) * scale).round().max(1.0) as u32;
    let resized = image.resize(next_w, next_h, image::imageops::FilterType::Lanczos3);
    let mut payload = Vec::new();
    resized.write_to(&mut Cursor::new(&mut payload), format)?;
    Ok((payload, Some(next_w), Some(next_h), false))
}

pub fn images_dir(site_json: &Path) -> PathBuf {
    site_json
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("source")
        .join("images")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    fn png(width: u32, height: u32) -> Vec<u8> {
        let image = ImageBuffer::from_pixel(width, height, Rgb([20, 40, 60]));
        let dynamic = image::DynamicImage::ImageRgb8(image);
        let mut bytes = Vec::new();
        dynamic
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        bytes
    }

    #[test]
    fn kebab_and_clash_rules() {
        assert_eq!(kebab_filename("My Photo.PNG"), "my-photo.png");
        assert_eq!(kebab_filename("already-kebab.webp"), "already-kebab.webp");
        assert_eq!(kebab_filename("hero (1).jpg"), "hero-1.jpg");
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("my-photo.png"), b"a").unwrap();
        assert_eq!(clash_name(dir.path(), "my-photo.png"), "my-photo-2.png");
        std::fs::write(dir.path().join("my-photo-2.png"), b"b").unwrap();
        assert_eq!(clash_name(dir.path(), "my-photo.png"), "my-photo-3.png");
    }

    #[test]
    fn copy_as_is_is_the_default_and_never_upscales() {
        let dir = tempfile::tempdir().unwrap();
        let small = png(32, 16);
        let imported = import_bytes("Shot.PNG", &small, dir.path(), true).unwrap();
        assert_eq!(imported.filename, "shot.png");
        assert_eq!(imported.stored, "assets/images/shot.png");
        assert!(imported.copied_as_is);
        assert_eq!(std::fs::read(dir.path().join("shot.png")).unwrap(), small);
        assert_eq!(imported.width, Some(32));

        let large = png(3000, 1000);
        let kept = import_bytes("wide.png", &large, dir.path(), false).unwrap();
        assert!(kept.copied_as_is);
        assert_eq!(std::fs::read(dir.path().join("wide.png")).unwrap(), large);
    }

    #[test]
    fn optional_downscale_stops_at_2560_and_keeps_png() {
        let dir = tempfile::tempdir().unwrap();
        let large = png(4000, 2000);
        let imported = import_bytes("banner.png", &large, dir.path(), true).unwrap();
        assert!(!imported.copied_as_is);
        assert_eq!(imported.width, Some(LONG_EDGE));
        assert_eq!(imported.height, Some(1280));
        let written = std::fs::read(dir.path().join("banner.png")).unwrap();
        let decoded = image::load_from_memory(&written).unwrap();
        use image::GenericImageView;
        assert_eq!(decoded.dimensions(), (LONG_EDGE, 1280));
        assert!(written.starts_with(&[0x89, b'P', b'N', b'G']));
    }

    #[test]
    fn gif_and_svg_stay_byte_for_byte_even_when_shrink_is_asked() {
        let dir = tempfile::tempdir().unwrap();
        let gif = b"GIF89a-not-a-real-frame".to_vec();
        let imported = import_bytes("Anim.GIF", &gif, dir.path(), true).unwrap();
        assert!(imported.copied_as_is);
        assert_eq!(std::fs::read(dir.path().join("anim.gif")).unwrap(), gif);
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#.to_vec();
        let imported = import_bytes("Logo.svg", &svg, dir.path(), true).unwrap();
        assert_eq!(std::fs::read(dir.path().join("logo.svg")).unwrap(), svg);
        assert!(imported.copied_as_is);
    }
}
