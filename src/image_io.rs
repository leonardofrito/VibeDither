use anyhow::Result;
use image::DynamicImage;
use std::path::Path;

pub fn load_from_path(path: &Path) -> Result<DynamicImage> {
    let img = image::open(path)?;
    Ok(img)
}

pub fn get_clipboard_image() -> Option<DynamicImage> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    let image_data = match clipboard.get_image() {
        Ok(img) => img,
        Err(_) => return None, // Silent fail if no image
    };
    
    // Convert arboard image to image crate DynamicImage
    let img = image::RgbaImage::from_raw(
        image_data.width as u32,
        image_data.height as u32,
        image_data.bytes.into_owned(),
    )?;
    Some(DynamicImage::ImageRgba8(img))
}
