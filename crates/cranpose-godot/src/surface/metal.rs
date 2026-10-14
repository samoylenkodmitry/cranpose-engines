//! The surface on Godot's Metal device. Cranpose submits on a queue of its
//! own, so frames cross between the two queues by GPU copies.

use cranpose::{GpuContext, GuestView, prelude::ExternalTexture};
use cranpose_gpu_interop::godot::{Alpha, GodotGpu, GodotTexture};
use godot::{
    classes::{
        Engine, RdTextureFormat, RdTextureView, RenderingDevice, RenderingServer, SubViewport,
        Texture2D, Texture2Drd,
        rendering_device::{DataFormat, TextureUsageBits},
    },
    prelude::*,
};

struct Output {
    rid: Rid,
    godot: GodotTexture,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

// A viewport texture, which Cranpose samples from a copy on its own queue.
struct Imported {
    rid: Rid,
    godot: GodotTexture,
    receiver: wgpu::Texture,
    texture: ExternalTexture,
    since: u64,
}

pub(super) struct Backend {
    gpu: GodotGpu,
    rd: Gd<RenderingDevice>,
    output: Option<Output>,
    display: Gd<Texture2Drd>,
    shown: Option<Imported>,
    candidate: Option<Imported>,
}

impl Backend {
    pub(super) fn new() -> Result<Self, String> {
        let mut rd = RenderingServer::singleton()
            .get_rendering_device()
            .ok_or("Godot renders without a RenderingDevice")?;
        let gpu = GodotGpu::new(&mut rd).map_err(|error| error.to_string())?;
        Ok(Self {
            gpu,
            rd,
            output: None,
            display: Texture2Drd::new_gd(),
            shown: None,
            candidate: None,
        })
    }

    pub(super) fn context(&self) -> &GpuContext {
        self.gpu.context()
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
        let ticket = self.gpu.begin_frame();
        let drawn = view.draw(&output.texture, &output.view);
        self.gpu.finish_frame(
            ticket,
            drawn.as_ref().is_ok_and(|drew| *drew),
            &output.texture,
            &output.godot,
        );
        drawn.map(|_| ()).map_err(|error| error.to_string())
    }

    // A viewport texture is new until Godot rendered into it: a frame later
    // it replaces the shown one, whose copy waits until then.
    pub(super) fn viewport_frame(
        &mut self,
        viewport: &mut Gd<SubViewport>,
        wanted: Vector2i,
    ) -> Option<ExternalTexture> {
        let now = Engine::singleton().get_process_frames();
        let rendered = viewport_texture(viewport);
        self.copy_rendered(rendered, now);
        if viewport.get_size() != wanted {
            viewport.set_size(wanted);
        }
        let next = viewport_texture(viewport)?;
        let known = |imported: &Option<Imported>| {
            imported
                .as_ref()
                .is_some_and(|imported| imported.rid == next)
        };
        if !known(&self.shown) && !known(&self.candidate) {
            match self.import(next, now) {
                Ok(imported) => self.candidate = Some(imported),
                Err(error) => {
                    godot_error!("Godot viewport texture: {error}");
                }
            }
        }
        self.shown.as_ref().map(|shown| shown.texture.clone())
    }

    fn copy_rendered(&mut self, current: Option<Rid>, now: u64) {
        let rendered = self
            .candidate
            .as_ref()
            .is_some_and(|candidate| candidate.since < now);
        if rendered {
            self.shown = self.candidate.take();
        } else if self
            .shown
            .as_ref()
            .is_none_or(|shown| Some(shown.rid) != current)
        {
            return;
        }
        if let Some(shown) = self.shown.as_ref() {
            self.gpu
                .receive(&shown.godot, &shown.receiver, Alpha::Opaque);
        }
    }

    fn import(&mut self, rid: Rid, since: u64) -> Result<Imported, String> {
        let godot = self
            .gpu
            .texture(&mut self.rd, rid)
            .map_err(|error| error.to_string())?;
        let receiver = self
            .gpu
            .receiver(&godot)
            .map_err(|error| error.to_string())?;
        let view = receiver.create_view(&wgpu::TextureViewDescriptor::default());
        let texture = ExternalTexture::new(receiver.width(), receiver.height(), true, view)
            .map_err(|error| error.to_string())?;
        Ok(Imported {
            rid,
            godot,
            receiver,
            texture,
            since,
        })
    }

    fn replace_output(&mut self, width: u32, height: u32) -> Result<(), String> {
        let mut format = RdTextureFormat::new_gd();
        format.set_width(width);
        format.set_height(height);
        format.set_format(DataFormat::R8G8B8A8_UNORM);
        format.set_usage_bits(
            TextureUsageBits::SAMPLING_BIT
                | TextureUsageBits::COLOR_ATTACHMENT_BIT
                | TextureUsageBits::CAN_COPY_TO_BIT,
        );
        let rid = self.rd.texture_create(&format, &RdTextureView::new_gd());
        if !rid.is_valid() {
            return Err("Godot could not create the output texture".into());
        }
        let godot = self
            .gpu
            .texture(&mut self.rd, rid)
            .map_err(|error| error.to_string())?;
        let texture = self.gpu.twin(&godot);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.display.set_texture_rd_rid(rid);
        if let Some(previous) = self.output.replace(Output {
            rid,
            godot,
            texture,
            view,
        }) {
            self.rd.free_rid(previous.rid);
        }
        Ok(())
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        if let Some(output) = self.output.take() {
            self.display.set_texture_rd_rid(Rid::Invalid);
            self.rd.free_rid(output.rid);
        }
    }
}

fn viewport_texture(viewport: &Gd<SubViewport>) -> Option<Rid> {
    let rid =
        RenderingServer::singleton().texture_get_rd_texture(viewport.get_texture()?.get_rid());
    rid.is_valid().then_some(rid)
}
