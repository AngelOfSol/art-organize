#[allow(dead_code)]
pub struct RawImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl RawImage {
    #[allow(dead_code)]
    pub fn make(raw: &[u8]) -> anyhow::Result<Self> {
        let image = image::load_from_memory(raw)?;
        let image = image.to_bgra8();
        let (width, height) = image.dimensions();
        let raw_data = image.into_raw();

        Ok(RawImage {
            data: raw_data,
            width,
            height,
        })
    }
}
