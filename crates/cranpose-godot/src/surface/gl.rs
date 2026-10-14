//! The surface on Godot's WebGL2 context in its web export. Cranpose draws
//! on the same context, so its texture and Godot's viewport textures pass
//! by their GL names, without copies.

use std::num::NonZeroU32;

use cranpose::{GpuContext, GuestView, prelude::ExternalTexture};
use cranpose_gpu_interop::gl;
use godot::{
    classes::{
        Image, ImageTexture, Os, RenderingServer, SubViewport, Texture2D, image::Format,
        rendering_server::TextureType,
    },
    prelude::*,
};

use super::FORMAT;

struct Output {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

// A viewport's color texture, which Cranpose samples where Godot drew it.
struct Borrowed {
    name: NonZeroU32,
    size: Vector2i,
    texture: ExternalTexture,
}

pub(super) struct Backend {
    gpu: GpuContext,
    display: Gd<ImageTexture>,
    output: Option<Output>,
    borrowed: Option<Borrowed>,
}

impl Backend {
    pub(super) fn new() -> Result<Self, String> {
        // A phone's tile-based GPU shades a page of unfolded liquid glass in
        // a large part of a second; the folded material fits in the frame.
        // Cranpose's own web shell decides this from the browser; under
        // Godot, its feature tags say where the page runs.
        let os = Os::singleton();
        if os.has_feature("web_android") || os.has_feature("web_ios") {
            cranpose::set_glass_material_folds(true);
        }
        let gpu = gl::current_context().map_err(|error| error.to_string())?;
        Ok(Self {
            gpu,
            display: placeholder()?,
            output: None,
            borrowed: None,
        })
    }

    pub(super) fn context(&self) -> &GpuContext {
        &self.gpu
    }

    pub(super) fn display(&self) -> Gd<Texture2D> {
        self.display.clone().upcast()
    }

    pub(super) fn draw(
        &mut self,
        view: &mut GuestView,
        width: u32,
        height: u32,
        resized: bool,
    ) -> Result<(), String> {
        if resized || self.output.is_none() {
            self.replace_output(width, height)?;
            view.shell().mark_dirty();
        }
        let Some(output) = self.output.as_ref() else {
            return Ok(());
        };
        let drawn = view.draw(&output.texture, &output.view);
        gl::reset_state(&self.gpu);
        drawn.map(|_| ()).map_err(|error| error.to_string())
    }

    pub(super) fn viewport_frame(
        &mut self,
        viewport: &mut Gd<SubViewport>,
        wanted: Vector2i,
    ) -> Option<ExternalTexture> {
        if viewport.get_size() != wanted {
            viewport.set_size(wanted);
        }
        // The viewport's `ViewportTexture` is a proxy without a GL name; its
        // render target's texture has one.
        let server = RenderingServer::singleton();
        let rid = server.viewport_get_texture(viewport.get_viewport_rid());
        let handle = server.texture_get_native_handle(rid);
        let name = NonZeroU32::new(u32::try_from(handle).ok()?)?;
        let size = viewport.get_size();
        if self
            .borrowed
            .as_ref()
            .is_none_or(|borrowed| borrowed.name != name || borrowed.size != size)
        {
            match self.borrow(name, size) {
                Ok(borrowed) => self.borrowed = Some(borrowed),
                Err(error) => {
                    godot_error!("Godot viewport texture: {error}");
                    return None;
                }
            }
        }
        self.borrowed
            .as_ref()
            .map(|borrowed| borrowed.texture.clone())
    }

    fn borrow(&self, name: NonZeroU32, size: Vector2i) -> Result<Borrowed, String> {
        let desc = wgpu::TextureDescriptor {
            label: Some("Godot viewport"),
            size: extent(size.x.max(1) as u32, size.y.max(1) as u32),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture =
            gl::borrow_texture(&self.gpu, name, &desc).map_err(|error| error.to_string())?;
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture = ExternalTexture::new(desc.size.width, desc.size.height, true, view)
            .map_err(|error| error.to_string())?;
        Ok(Borrowed {
            name,
            size,
            texture,
        })
    }

    // Godot's display texture takes the new texture's GL name; the old
    // texture goes once Godot no longer refers to it.
    fn replace_output(&mut self, width: u32, height: u32) -> Result<(), String> {
        let texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Cranpose surface"),
            size: extent(width, height),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let name = gl::texture_name(&texture).ok_or("the surface texture has no GL name")?;
        let mut server = RenderingServer::singleton();
        let native = server.texture_create_from_native_handle(
            TextureType::TYPE_2D,
            Format::RGBA8,
            name.get().into(),
            width as i32,
            height as i32,
            1,
        );
        // Godot frees `native` once its texture moved into the display's.
        server.texture_replace(self.display.get_rid(), native);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.output = Some(Output { texture, view });
        Ok(())
    }
}

impl Drop for Backend {
    // Godot's display texture may outlive the surface's GL texture.
    fn drop(&mut self) {
        if self.output.is_some()
            && let Some(image) = Image::create_empty(1, 1, false, Format::RGBA8)
        {
            let mut server = RenderingServer::singleton();
            let empty = server.texture_2d_create(&image);
            server.texture_replace(self.display.get_rid(), empty);
        }
    }
}

fn placeholder() -> Result<Gd<ImageTexture>, String> {
    let image = Image::create_empty(1, 1, false, Format::RGBA8).ok_or("Godot made no image")?;
    ImageTexture::create_from_image(&image).ok_or_else(|| "Godot made no texture".to_string())
}

fn extent(width: u32, height: u32) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    }
}
