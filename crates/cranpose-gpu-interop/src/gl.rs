//! Cranpose on the WebGL2 context another module made current under
//! Emscripten, such as a Godot web export's. Both draw on one context, so a
//! texture one draws the other samples by its GL name, without a copy.

use std::{num::NonZeroU32, sync::Arc};

use cranpose::GpuContext;
use glow::HasContext;
use wgpu_hal::{api::Gles, gles::TextureInner};

use crate::{InteropError, gl_names::gl_proc_address};

// Texture units whose bindings Cranpose's frames may leave behind.
const TEXTURE_UNITS: u32 = 16;

/// Cranpose's GPU context on the GL context current on this thread.
pub fn current_context() -> Result<GpuContext, InteropError> {
    // wgpu's own GL instance would open a context through EGL; this one has
    // no backend and only wraps the host's context.
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::empty(),
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    // SAFETY: the host's context is current on this thread, the one thread of
    // an Emscripten module without threads, so it stays current whenever
    // wgpu calls GL. The loader returns the module's own WebGL entry points.
    let exposed = unsafe {
        wgpu_hal::gles::Adapter::new_external(gl_proc_address, wgpu::GlBackendOptions::default())
    }
    .ok_or_else(|| InteropError::NoAdapter("WebGL2".into()))?;
    // SAFETY: the adapter comes from the GL backend this wgpu is built with.
    let adapter = unsafe { instance.create_adapter_from_hal(exposed) };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("Cranpose"),
        required_limits: adapter.limits(),
        ..Default::default()
    }))?;
    Ok(GpuContext {
        instance,
        adapter,
        device: Arc::new(device),
        queue: Arc::new(queue),
    })
}

/// The GL name of `texture`, a texture of [`current_context`], for the host
/// to sample or draw into.
pub fn texture_name(texture: &wgpu::Texture) -> Option<NonZeroU32> {
    // SAFETY: only reads the name of a texture of the GL backend.
    let hal = unsafe { texture.as_hal::<Gles>() }?;
    match &hal.inner {
        TextureInner::Texture { raw, .. } => Some(raw.0),
        _ => None,
    }
}

/// The host's GL texture `name` as a texture of `gpu` described by `desc`.
/// The host keeps owning it: wgpu never deletes it. A name the host deleted
/// meanwhile makes WebGL report errors, not touch other memory.
pub fn borrow_texture(
    gpu: &GpuContext,
    name: NonZeroU32,
    desc: &wgpu::TextureDescriptor,
) -> Result<wgpu::Texture, InteropError> {
    let hal_desc = wgpu_hal::TextureDescriptor {
        label: desc.label,
        size: desc.size,
        mip_level_count: desc.mip_level_count,
        sample_count: desc.sample_count,
        dimension: desc.dimension,
        format: desc.format,
        usage: wgpu::TextureUses::RESOURCE | wgpu::TextureUses::COPY_SRC,
        memory_flags: wgpu_hal::MemoryFlags::empty(),
        view_formats: Vec::new(),
    };
    // SAFETY: the device is of the GL backend. The name is a live 2D texture
    // of the same context with `desc`'s size and format, as the host reports
    // them, and the empty drop callback leaves its deletion to the host.
    unsafe {
        let device = gpu
            .device
            .as_hal::<Gles>()
            .ok_or(InteropError::MissingHandle("GL device"))?;
        let texture = device.texture_from_raw(name, &hal_desc, Some(Box::new(|| {})));
        // GL has no layouts; the state only tells wgpu the host wrote the
        // texture, so it never clears it.
        Ok(gpu
            .device
            .create_texture_from_hal::<Gles>(texture, desc, wgpu::TextureUses::RESOURCE))
    }
}

/// Puts back the GL state a host expects after Cranpose drew on its context:
/// no framebuffer, program, vertex array, buffer, sampler or texture bound,
/// the fixed-function tests off, every write mask open, and the pixel
/// unpacking parameters at their defaults.
pub fn reset_state(gpu: &GpuContext) {
    // SAFETY: the adapter is of the GL backend.
    let Some(adapter) = (unsafe { gpu.adapter.as_hal::<Gles>() }) else {
        return;
    };
    let gl = adapter.adapter_context().lock();
    // SAFETY: plain state changes on the current context; each binds nothing
    // or sets a default, which every GL object stays valid under.
    unsafe {
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        gl.bind_renderbuffer(glow::RENDERBUFFER, None);
        gl.use_program(None);
        gl.bind_vertex_array(None);
        for target in [
            glow::ARRAY_BUFFER,
            glow::UNIFORM_BUFFER,
            glow::PIXEL_PACK_BUFFER,
            glow::PIXEL_UNPACK_BUFFER,
            glow::COPY_READ_BUFFER,
            glow::COPY_WRITE_BUFFER,
        ] {
            gl.bind_buffer(target, None);
        }
        for unit in 0..TEXTURE_UNITS {
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_sampler(unit, None);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.bind_texture(glow::TEXTURE_2D_ARRAY, None);
            gl.bind_texture(glow::TEXTURE_3D, None);
            gl.bind_texture(glow::TEXTURE_CUBE_MAP, None);
        }
        gl.active_texture(glow::TEXTURE0);
        for capability in [
            glow::SCISSOR_TEST,
            glow::BLEND,
            glow::DEPTH_TEST,
            glow::CULL_FACE,
            glow::STENCIL_TEST,
            glow::POLYGON_OFFSET_FILL,
            glow::SAMPLE_ALPHA_TO_COVERAGE,
            glow::RASTERIZER_DISCARD,
        ] {
            gl.disable(capability);
        }
        gl.front_face(glow::CCW);
        gl.color_mask(true, true, true, true);
        gl.depth_mask(true);
        gl.stencil_mask(!0);
        gl.blend_equation(glow::FUNC_ADD);
        gl.blend_func(glow::ONE, glow::ZERO);
        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
        gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, 0);
        gl.pixel_store_i32(glow::UNPACK_IMAGE_HEIGHT, 0);
        gl.pixel_store_i32(glow::UNPACK_SKIP_ROWS, 0);
        gl.pixel_store_i32(glow::UNPACK_SKIP_PIXELS, 0);
        gl.pixel_store_i32(glow::UNPACK_SKIP_IMAGES, 0);
        gl.pixel_store_i32(glow::PACK_ALIGNMENT, 4);
    }
}
