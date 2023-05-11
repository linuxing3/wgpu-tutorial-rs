use anyhow::*;
use image::{DynamicImage, GenericImageView, RgbaImage};
use imgui_wgpu::TextureConfig;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

// impl Copy for image::DynamicImage {
//     // add code here
//     fn clone(&mut self) -> image::DynamicImage {}
// }

impl Texture {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;

        Self::from_image(device, queue, &img, Some(label))
    }

    fn resize_image(
        data: RgbaImage,
        width: u32,
        height: u32,
    ) -> RgbaImage {
        // TODO:

        DynamicImage::ImageRgba8(data)
            .resize(width, height, image::imageops::FilterType::Triangle)
            .into_rgba8()
    }

    pub fn imgui_image_from_raw<'a>(bytes: &'a [u8]) -> (image::DynamicImage, wgpu::Extent3d) {
        let image = image::load_from_memory(bytes).expect("load from raw failed!");

        let (width, height) = image.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            ..Default::default()
        };

        (image, size)
    }

    pub fn imgui_texture_from_raw<'a>(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        renderer: &'a imgui_wgpu::Renderer,
        image: &image::DynamicImage,
        size: wgpu::Extent3d,
    ) -> imgui_wgpu::Texture {
        let rgba = image.to_rgba8();

        let raw_data = rgba.into_raw();

        let texture_config = imgui_wgpu::TextureConfig {
            size,
            label: Some("raw texture"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        };

        let texture = imgui_wgpu::Texture::new(&device, &renderer, texture_config);

        texture.write(&queue, &raw_data, size.width, size.height);

        texture
    }

    pub fn imgui_texture_from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &imgui_wgpu::Renderer,
        bytes: &[u8],
        format: image::ImageFormat,
    ) -> (Vec<u8>, imgui_wgpu::Texture) {
        let image =
            image::load_from_memory_with_format(bytes, format).expect("invalid image_format");

        let rgba = image.to_rgba8();

        // NOTE: raw data with rgba8 format
        let raw_data = rgba.into_raw();

        let dimensions = image.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            ..Default::default()
        };

        let texture_config = imgui_wgpu::TextureConfig {
            size,
            label: Some("image texture"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        };

        let texture = imgui_wgpu::Texture::new(&device, &renderer, texture_config);

        texture.write(&queue, &raw_data, dimensions.0, dimensions.1);

        (raw_data, texture)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();

        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}
