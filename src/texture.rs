use anyhow::*;
use image::{DynamicImage, GenericImageView, RgbaImage};
use std::num::NonZeroU32;

pub struct Context<'a> {
    pub device : &'a wgpu::Device,
    pub queue : &'a wgpu::Queue,
    pub renderer : &'a mut imgui_wgpu::Renderer, // NOTE: must be mutable
}

pub struct Texture {
    pub texture : wgpu::Texture,
    pub view : wgpu::TextureView,
    pub sampler : wgpu::Sampler,
}

// impl Copy for image::DynamicImage {
//     // add code here
//     fn clone(&mut self) -> image::DynamicImage {}
// }

impl Texture {
    pub const DEPTH_FORMAT : wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    // 1.

    pub fn create_depth_texture(
        device : &wgpu::Device,
        config : &wgpu::SurfaceConfiguration,
        label : &str,
    ) -> Self {

        let size = wgpu::Extent3d {
            width : config.width,
            height : config.height,
            depth_or_array_layers : 1,
        };

        let desc = wgpu::TextureDescriptor {
            label : Some(label),
            size,
            mip_level_count : 1,
            sample_count : 1,
            dimension : wgpu::TextureDimension::D2,
            format : Self::DEPTH_FORMAT,
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats : &[Self::DEPTH_FORMAT],
        };

        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u : wgpu::AddressMode::ClampToEdge,
            address_mode_v : wgpu::AddressMode::ClampToEdge,
            address_mode_w : wgpu::AddressMode::ClampToEdge,
            mag_filter : wgpu::FilterMode::Linear,
            min_filter : wgpu::FilterMode::Linear,
            mipmap_filter : wgpu::FilterMode::Nearest,
            compare : Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp : 0.0,
            lod_max_clamp : 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn new_texture(context : &mut Context, size : [f32; 2]) -> imgui::TextureId {

        // Stores a texture for displaying with imgui::Image(),
        // also as a texture view for rendering into it

        let texture_config = imgui_wgpu::TextureConfig {
            size : wgpu::Extent3d {
                width : size[0] as u32,
                height : size[1] as u32,
                ..Default::default()
            },
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ..Default::default()
        };

        let texture = imgui_wgpu::Texture::new(&context.device, &context.renderer, texture_config);

        let texture_id = context.renderer.textures.insert(texture);

        texture_id
    }

    pub fn from_bytes(bytes : &[u8], context : &Context, label : &str) -> Result<Self> {

        let img = image::load_from_memory(bytes)?;

        Self::from_image(context, &img, Some(label))
    }

    #[allow(dead_code)]

    pub fn from_bytes_with_label(
        device : &wgpu::Device,
        queue : &wgpu::Queue,
        bytes : &[u8],
        label : &str,
    ) -> Result<Self> {

        let img = image::load_from_memory(bytes)?;

        Self::from_image_with_label(device, queue, &img, Some(label))
    }

    pub fn from_image_with_label(
        device : &wgpu::Device,
        queue : &wgpu::Queue,
        img : &image::DynamicImage,
        label : Option<&str>,
    ) -> Result<Self> {

        let dimensions = img.dimensions();

        let rgba = img.to_rgba8();

        let size = wgpu::Extent3d {
            width : dimensions.0,
            height : dimensions.1,
            depth_or_array_layers : 1,
        };

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count : 1,
            sample_count : 1,
            dimension : wgpu::TextureDimension::D2,
            format,
            usage : wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats : &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect : wgpu::TextureAspect::All,
                texture : &texture,
                mip_level : 0,
                origin : wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset : 0,
                bytes_per_row : NonZeroU32::new(4 * dimensions.0),
                rows_per_image : NonZeroU32::new(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u : wgpu::AddressMode::ClampToEdge,
            address_mode_v : wgpu::AddressMode::ClampToEdge,
            address_mode_w : wgpu::AddressMode::ClampToEdge,
            mag_filter : wgpu::FilterMode::Linear,
            min_filter : wgpu::FilterMode::Nearest,
            mipmap_filter : wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub fn recreate_image(
        context : &mut Context,
        texture_size : u32,
        texture_texels : &Vec<u8>,
    ) -> imgui_wgpu::Texture {

        let cube_texture_extent = wgpu::Extent3d {
            width : texture_size,
            height : texture_size,
            depth_or_array_layers : 1,
        };

        let cube_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label : None,
            size : cube_texture_extent,
            mip_level_count : 1,
            sample_count : 1,
            dimension : wgpu::TextureDimension::D2,
            format : wgpu::TextureFormat::R8Uint,
            usage : wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats : &[wgpu::TextureFormat::R8Uint],
        });

        //NOTE:
        context.queue.write_texture(
            cube_texture.as_image_copy(),
            &texture_texels,
            wgpu::ImageDataLayout {
                offset : 0,
                bytes_per_row : Some(std::num::NonZeroU32::new(texture_size).unwrap()),
                rows_per_image : None, // NOTE: None for pixels from scratch
            },
            cube_texture_extent,
        );

        let texture_config = imgui_wgpu::TextureConfig {
            size : wgpu::Extent3d {
                width : texture_size,
                height : texture_size,
                ..Default::default()
            },
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ..Default::default()
        };

        imgui_wgpu::Texture::new(context.device, context.renderer, texture_config)
    }

    pub fn create_texels(size : usize) -> Vec<u8> {

        (0..size * size)
            .map(|id| {

                // get high five for recognizing this ;)
                let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;

                let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;

                let (mut x, mut y, mut count) = (cx, cy, 0);

                while count < 0xFF && x * x + y * y < 4.0 {

                    let old_x = x;

                    x = x * x - y * y + cx;

                    y = 2.0 * old_x * y + cy;

                    count += 1;
                }

                count
            })
            .collect()
    }

    fn resize_image(data : RgbaImage, width : u32, height : u32) -> RgbaImage {

        // TODO:

        DynamicImage::ImageRgba8(data)
            .resize(width, height, image::imageops::FilterType::Triangle)
            .into_rgba8()
    }

    pub fn imgui_image_from_raw<'a>(bytes : &'a [u8]) -> (image::DynamicImage, wgpu::Extent3d) {

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
        context : &Context,
        image : &image::DynamicImage,
        size : wgpu::Extent3d,
    ) -> imgui_wgpu::Texture {

        let rgba = image.to_rgba8();

        let raw_data = rgba.into_raw();

        let texture_config = imgui_wgpu::TextureConfig {
            size,
            label : Some("raw texture"),
            format : Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        };

        let texture = imgui_wgpu::Texture::new(context.device, context.renderer, texture_config);

        // NOTE: queue.write?
        texture.write(&context.queue, &raw_data, size.width, size.height);

        texture
    }

    pub fn imgui_texture_from_image(
        context : &Context,
        bytes : &[u8],
        format : image::ImageFormat,
    ) -> (Vec<u8>, imgui_wgpu::Texture) {

        let image =
            image::load_from_memory_with_format(bytes, format).expect("invalid image_format");

        let rgba = image.to_rgba8();

        // NOTE: raw data with rgba8 format
        let raw_data = rgba.into_raw();

        let (width, height) = image.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            ..Default::default()
        };

        let texture_config = imgui_wgpu::TextureConfig {
            size,
            label : Some("image texture"),
            format : Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        };

        let texture = imgui_wgpu::Texture::new(context.device, context.renderer, texture_config);

        texture.write(&context.queue, &raw_data, width, height);

        (raw_data, texture)
    }

    pub fn from_image(
        context : &Context,
        img : &image::DynamicImage,
        label : Option<&str>,
    ) -> Result<Self> {

        let rgba = img.to_rgba8();

        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width : dimensions.0,
            height : dimensions.1,
            depth_or_array_layers : 1,
        };

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count : 1,
            sample_count : 1,
            dimension : wgpu::TextureDimension::D2,
            format : wgpu::TextureFormat::Rgba8UnormSrgb,
            usage : wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats : &[],
        });

        context.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect : wgpu::TextureAspect::All,
                texture : &texture,
                mip_level : 0,
                origin : wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset : 0,
                bytes_per_row : std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image : std::num::NonZeroU32::new(dimensions.1), // NOTE: None?
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u : wgpu::AddressMode::ClampToEdge,
            address_mode_v : wgpu::AddressMode::ClampToEdge,
            address_mode_w : wgpu::AddressMode::ClampToEdge,
            mag_filter : wgpu::FilterMode::Linear,
            min_filter : wgpu::FilterMode::Nearest,
            mipmap_filter : wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}

#[cfg(test)]

mod test {

    use super::*;

    #[test]

    pub fn test_dimension() {

        let bytes = include_bytes!("../assets/images/happy-tree.png");

        let (image, size) = Texture::imgui_image_from_raw(bytes);

        println!("{}", size.height);

        println!("{}", size.width);

        assert!(Some(size.height) != None);
    }
}
