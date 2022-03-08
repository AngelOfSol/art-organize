use db::BlobId;
use egui::TextureId;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
};
use wgpu::{util::DeviceExt, Device, Extent3d, Queue, TextureDescriptor};

use crate::backend::DbBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImageRequest {
    pub request_type: ImageRequestType,

    blob_id: BlobId,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImageRequestType {
    Thumbnail,
    Image,
}

pub struct ImageData {
    pub image: BTreeMap<ImageRequest, Image>,
    outgoing: ImageRequester,
    incoming: ImageDataReceiver,
}

impl ImageData {
    pub fn image_for(&self, blob_id: BlobId, db: &DbBackend) -> ImageStatus {
        let request = ImageRequest {
            blob_id,
            request_type: ImageRequestType::Image,
        };
        self.request(request, db, blob_id)
    }
    pub fn thumbnail_for(&self, blob_id: BlobId, db: &DbBackend) -> ImageStatus {
        let request = ImageRequest {
            blob_id,
            request_type: ImageRequestType::Thumbnail,
        };
        self.request(request, db, blob_id)
    }

    fn request(&self, request: ImageRequest, db: &DbBackend, blob_id: BlobId) -> ImageStatus {
        if let Some(image) = self.image.get(&request).copied() {
            ImageStatus::Available(image)
        } else {
            let _ = self
                .outgoing
                .send((request, db.storage_for(blob_id)))
                .unwrap();
            ImageStatus::Unavailable
        }
    }

    pub fn add_image(&mut self, request: ImageRequest, image: Image) {
        self.image.insert(request, image);
    }

    pub fn create_textures(
        &mut self,
        egui_rpass: &mut egui_wgpu_backend::RenderPass,
        queue: &Queue,
        device: &Device,
    ) {
        while let Ok((request, image)) = self.incoming.try_recv() {
            let image_texture_id = make_texture(device, queue, &image, egui_rpass);
            self.add_image(
                request,
                Image {
                    id: image_texture_id,
                    width: image.width as u32,
                    height: image.height as u32,
                },
            );
        }
    }
}

pub type ImageRequester = mpsc::Sender<(ImageRequest, PathBuf)>;
pub type ImageDataReceiver = mpsc::Receiver<(ImageRequest, RawImage)>;

pub struct TextureLoadingTask {
    incoming: mpsc::Receiver<(ImageRequest, PathBuf)>,
    outgoing: mpsc::Sender<(ImageRequest, RawImage)>,
}

impl TextureLoadingTask {
    pub fn run() -> ImageData {
        let (send_request, recv_request) = mpsc::channel();
        let (send_image, recv_image) = mpsc::channel();

        let handle = TextureLoadingTask {
            incoming: recv_request,
            outgoing: send_image,
        };

        tokio::task::spawn_blocking(move || {
            let mut load_requested = BTreeSet::new();
            while let Ok((request, path)) = handle.incoming.recv() {
                if !load_requested.contains(&request) {
                    load_requested.insert(request);

                    let send_image = handle.outgoing.clone();

                    tokio::spawn(async move {
                        let image_data = tokio::fs::read(&path).await?;

                        use image::GenericImageView as _;
                        let image = image::load_from_memory(&image_data)?;

                        let image = match request.request_type {
                            ImageRequestType::Thumbnail => image.thumbnail(256, 256),
                            ImageRequestType::Image => image,
                        };

                        let image = RawImage {
                            width: image.width(),
                            height: image.height(),
                            data: image.to_rgba8().to_vec(),
                        };

                        send_image.send((request, image))?;

                        anyhow::Result::<()>::Ok(())
                    });
                }
            }
        });
        ImageData {
            image: Default::default(),
            outgoing: send_request,
            incoming: recv_image,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ImageStatus {
    Loading,
    Unavailable,
    Available(Image),
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
    pub fn with_height(&self, height: f32) -> [f32; 2] {
        let size = [self.width as f32, self.height as f32];
        let aspect_ratio = size[0] / size[1];

        [height * aspect_ratio, height]
    }
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
