use db::BlobId;
use egui::TextureId;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use wgpu::{util::DeviceExt, Device, Extent3d, Queue, TextureDescriptor};

use crate::backend::DbBackend;

#[derive(Clone)]
pub struct TextureStorage {
    pub inner: Arc<Mutex<TextureStorageHandle>>,
}

#[derive(Copy, Clone, Debug)]
pub enum ImageStatus {
    Loading,
    Unavailable,
    Available(Image),
}

impl TextureStorage {
    pub fn new() -> Self {
        TextureStorage {
            inner: Arc::new(Mutex::new(TextureStorageHandle {
                loading: Default::default(),
                loaded: Default::default(),
                image: Default::default(),
                thumbnail: Default::default(),
                unavailable: Default::default(),
            })),
        }
    }

    pub fn image_for(&self, blob_id: BlobId, db: &DbBackend) -> ImageStatus {
        let mut inner = if let Ok(inner) = self.inner.try_lock() {
            inner
        } else {
            return ImageStatus::Loading;
        };

        if let Some(image) = inner.image.get(&blob_id).copied() {
            ImageStatus::Available(image)
        } else if inner.unavailable.contains(&blob_id) {
            ImageStatus::Unavailable
        } else {
            if inner.loading.insert(blob_id) {
                let inner = self.inner.clone();
                let storage = db.storage_for(blob_id);
                tokio::spawn(load(blob_id, storage, inner));
            }

            ImageStatus::Loading
        }
    }

    pub fn thumbnail_for(&self, blob_id: BlobId, db: &DbBackend) -> ImageStatus {
        let mut inner = if let Ok(inner) = self.inner.try_lock() {
            inner
        } else {
            return ImageStatus::Loading;
        };

        if let Some(image) = inner.thumbnail.get(&blob_id).copied() {
            ImageStatus::Available(image)
        } else if inner.unavailable.contains(&blob_id) {
            ImageStatus::Unavailable
        } else {
            if inner.loading.insert(blob_id) {
                let inner = self.inner.clone();
                let storage = db.storage_for(blob_id);
                tokio::spawn(load(blob_id, storage, inner));
            }

            ImageStatus::Loading
        }
    }
}

async fn load(blob_id: BlobId, storage: PathBuf, inner: Arc<Mutex<TextureStorageHandle>>) {
    let _load_result = async {
        let image_data = tokio::fs::read(storage).await?;
        use image::GenericImageView as _;
        let image = if let Ok(image) = image::load_from_memory(&image_data) {
            image
        } else {
            let mut inner = inner.lock().unwrap();

            inner.loading.remove(&blob_id);
            inner.unavailable.insert(blob_id);

            return Ok(());
        };

        let thumbnail = image.thumbnail(256, 256);
        let thumbnail = RawImage {
            width: thumbnail.width(),
            height: thumbnail.height(),
            data: thumbnail.to_rgba8().to_vec(),
        };

        let image = RawImage {
            width: image.width(),
            height: image.height(),
            data: image.to_rgba8().to_vec(),
        };

        let mut inner = inner.lock().unwrap();

        inner.loading.remove(&blob_id);
        inner.loaded.insert(blob_id, (thumbnail, image));

        anyhow::Result::<()>::Ok(())
    }
    .await;
}

pub struct TextureStorageHandle {
    loading: BTreeSet<BlobId>,
    loaded: BTreeMap<BlobId, (RawImage, RawImage)>,
    unavailable: BTreeSet<BlobId>,
    image: BTreeMap<BlobId, Image>,
    thumbnail: BTreeMap<BlobId, Image>,
}

#[derive(Clone, Debug)]
pub struct RawImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Image {
    pub id: TextureId,
    pub width: u32,
    pub height: u32,
}

impl Image {
    pub fn scaled(&self, max_size: [f32; 2]) -> [f32; 2] {
        let zoom = 1.0;

        let size = [self.width as f32 * zoom, self.height as f32 * zoom];
        let aspect_ratio = size[0] / size[1];
        let new_aspect_ratio = max_size[0] / max_size[1];

        if size[0] <= max_size[0] && size[1] <= max_size[1] {
            size
        } else {
            let use_width = aspect_ratio >= new_aspect_ratio;

            if use_width {
                [max_size[0], size[1] * max_size[0] / size[0]]
            } else {
                [size[0] * max_size[1] / size[1], max_size[1]]
            }
        }
    }
}

impl TextureStorageHandle {
    pub fn create_textures(
        &mut self,
        egui_rpass: &mut egui_wgpu_backend::RenderPass,
        queue: &Queue,
        device: &Device,
    ) {
        let loaded = std::mem::take(&mut self.loaded);

        for (blob_id, (thumbnail, image)) in loaded {
            let image_texture_id = make_texture(device, queue, &image, egui_rpass);
            let thumbnail_texture_id = make_texture(device, queue, &thumbnail, egui_rpass);

            self.loading.remove(&blob_id);
            self.image.insert(
                blob_id,
                Image {
                    id: image_texture_id,
                    width: image.width as u32,
                    height: image.height as u32,
                },
            );
            self.thumbnail.insert(
                blob_id,
                Image {
                    id: thumbnail_texture_id,
                    width: image.width as u32,
                    height: image.height as u32,
                },
            );
        }
    }
}

fn make_texture(
    device: &Device,
    queue: &Queue,
    image: &RawImage,
    egui_rpass: &mut egui_wgpu_backend::RenderPass,
) -> TextureId {
    let texture = device.create_texture_with_data(
        queue,
        &TextureDescriptor {
            label: None,
            size: Extent3d {
                width: image.width,
                height: image.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        },
        &image.data,
    );

    egui_rpass.egui_texture_from_wgpu_texture(device, &texture, wgpu::FilterMode::Linear)
}
