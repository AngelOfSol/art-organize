use db::BlobId;
use egui::{FontImage, TextureId};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};
use wgpu::{util::DeviceExt, Device, Extent3d, Queue, TextureDescriptor};

use crate::backend::DbBackend;

#[derive(Clone)]
pub struct TextureStorage {
    pub inner: Arc<Mutex<TextureStorageHandle>>,
}

impl TextureStorage {
    pub fn new() -> Self {
        TextureStorage {
            inner: Arc::new(Mutex::new(TextureStorageHandle {
                loading: Default::default(),
                loaded: Default::default(),
                data: Default::default(),
            })),
        }
    }

    pub fn get(&self, blob_id: BlobId, db: &DbBackend) -> Option<Image> {
        let mut inner = self.inner.try_lock().ok()?;

        if let image @ Some(_) = inner.data.get(&blob_id).copied() {
            image
        } else {
            if inner.loading.insert(blob_id) {
                let inner = self.inner.clone();
                let storage = db.storage_for(blob_id);
                tokio::spawn(async move {
                    let x = async {
                        let image_data = tokio::fs::read(storage).await?;
                        use image::GenericImageView as _;
                        let image = image::load_from_memory(&image_data)?;

                        let image = FontImage {
                            version: 0,
                            width: image.width() as usize,
                            height: image.height() as usize,
                            pixels: image.to_rgba8().to_vec(),
                        };

                        let inner = inner.lock();

                        if inner.is_err() {
                            dbg!(unsafe { inner.unwrap_err_unchecked() });
                            return Ok(());
                        }

                        let mut inner = inner.unwrap();

                        inner.loading.remove(&blob_id);
                        inner.loaded.insert(blob_id, image);

                        anyhow::Result::<()>::Ok(())
                    }
                    .await;
                    if x.is_err() {
                        let _ = dbg!(x);
                    }
                });
            }

            None
        }
    }
}

pub struct TextureStorageHandle {
    loading: BTreeSet<BlobId>,
    loaded: BTreeMap<BlobId, FontImage>,
    data: BTreeMap<BlobId, Image>,
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

        for (blob_id, image) in loaded {
            let texture = device.create_texture_with_data(
                queue,
                &TextureDescriptor {
                    label: None,
                    size: Extent3d {
                        width: image.width as u32,
                        height: image.height as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                },
                &image.pixels,
            );
            let texture_id = egui_rpass.egui_texture_from_wgpu_texture(
                device,
                &texture,
                wgpu::FilterMode::Nearest,
            );

            self.loading.remove(&blob_id);
            self.data.insert(
                blob_id,
                Image {
                    id: texture_id,
                    width: image.width as u32,
                    height: image.height as u32,
                },
            );
        }
    }
}
