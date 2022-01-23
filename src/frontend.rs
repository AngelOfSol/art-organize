pub struct Frontend {}

impl Frontend {
    pub fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        // ctx.
        //
    }
}

fn load_image(image_data: &[u8]) -> Result<epi::Image, image::ImageError> {
    use image::GenericImageView as _;
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(epi::Image::from_rgba_unmultiplied(size, pixels.as_slice()))
}
