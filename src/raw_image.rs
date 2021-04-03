use crate::consts::THUMBNAIL_SIZE;
use imgui::TextureId;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageGeneric<T> {
    pub data: T,
    pub width: u32,
    pub height: u32,
    pub hash: u64,
}

pub type RawImage = ImageGeneric<Vec<u8>>;
pub type TextureImage = ImageGeneric<TextureId>;

impl RawImage {
    pub fn make(path: PathBuf, hash: u64) -> anyhow::Result<(Self, Self)> {
        let image = image::io::Reader::open(&path)?
            .with_guessed_format()?
            .decode()?;
        let thumbnail = image.thumbnail(THUMBNAIL_SIZE as u32, THUMBNAIL_SIZE as u32);

        let image = image.to_bgra8();
        let (width, height) = image.dimensions();

        let raw = RawImage {
            data: image.into_raw(),
            width,
            height,
            hash,
        };

        let thumbnail = thumbnail.into_bgra8();
        let (width, height) = thumbnail.dimensions();

        let thumbnail = RawImage {
            data: thumbnail.into_raw(),
            width,
            height,
            hash,
        };
        Ok((raw, thumbnail))
    }
}
