use std::sync::Arc;

use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    core_pipeline::{FullscreenShader, schedule::Core3d, upscaling::upscaling},
    prelude::*,
    render::{
        GpuResourceAppExt, Render, RenderApp, RenderStartup, RenderSystems,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            BlendState, CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState,
            PipelineCache, RenderPassDescriptor, RenderPipelineDescriptor, ShaderStages,
            SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat,
            TextureSampleType, TextureView, TextureViewId, binding_types::texture_2d,
        },
        renderer::{RenderContext, ViewQuery},
        view::ViewTarget,
    },
    shader::Shader,
};
use cranpose::{AppLauncher, GpuContext, GuestView};

use crate::{ContentFactory, gpu::CranposeGpu};

const HUD_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Draws a Cranpose composition over the output of the camera marked with
/// [`HudCamera`], at that camera's target size.
///
/// Cranpose composes and draws on Bevy's main thread, into one of two
/// textures, and only when the composition changed. Bevy's render world
/// blends the other one, the newest finished HUD, over the camera's final
/// image. Pointer, touch, wheel and key input reach the HUD before the
/// screens in the world and the game; [`crate::CranposeInput`] says what it
/// took.
pub struct CranposeHudPlugin {
    content: Arc<ContentFactory>,
}

/// Marks the camera that a [`CranposeHudPlugin`] draws over.
#[derive(Component, Clone, Copy, Default, ExtractComponent)]
#[extract_app(RenderApp)]
pub struct HudCamera;

#[derive(Resource)]
struct HudContent(Arc<ContentFactory>);

#[derive(Clone)]
struct HudTexture {
    texture: wgpu::Texture,
    view: TextureView,
}

// The newest finished HUD, which the render world blends.
#[derive(Resource, Clone, Default, ExtractResource)]
#[extract_app(RenderApp)]
struct HudFront(Option<HudTexture>);

/// The HUD's composition, on Bevy's main thread.
pub(crate) struct HudView {
    pub(crate) view: GuestView,
    textures: [Option<HudTexture>; 2],
    front: Option<usize>,
}

#[derive(Resource)]
struct HudPipeline {
    layout: BindGroupLayoutDescriptor,
    fullscreen: FullscreenShader,
    shader: Handle<Shader>,
}

#[derive(Component)]
struct ViewHudPipeline {
    id: CachedRenderPipelineId,
    format: TextureFormat,
}

impl CranposeHudPlugin {
    /// `content` gets Bevy's world once and returns the composition's root.
    pub fn new(content: impl Fn(&mut World) -> Box<dyn FnMut()> + Send + Sync + 'static) -> Self {
        Self {
            content: Arc::new(content),
        }
    }
}

impl Plugin for CranposeHudPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "hud.wgsl");
        crate::input::add_to(app);
        app.insert_resource(HudContent(Arc::clone(&self.content)))
            .init_resource::<HudFront>()
            .add_plugins((
                ExtractComponentPlugin::<HudCamera>::default(),
                ExtractResourcePlugin::<HudFront>::default(),
            ))
            .add_systems(Last, draw_hud);
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            render
                .init_gpu_resource::<SpecializedRenderPipelines<HudPipeline>>()
                .add_systems(RenderStartup, init_hud_pipeline)
                .add_systems(Render, prepare_hud_pipelines.in_set(RenderSystems::Prepare))
                .add_systems(Core3d, hud_pass.after(upscaling));
        }
    }

    fn finish(&self, app: &mut App) {
        crate::gpu::insert(app);
    }
}

fn draw_hud(world: &mut World) {
    let Some((size, density)) = world
        .query_filtered::<&Camera, With<HudCamera>>()
        .iter(world)
        .find_map(|camera| {
            Some((
                camera.physical_target_size()?,
                camera.target_scaling_factor()?,
            ))
        })
    else {
        return;
    };
    if size.x == 0 || size.y == 0 || !ensure_view(world, density) {
        return;
    }
    let Some(gpu) = world.get_resource::<CranposeGpu>().map(|gpu| gpu.0.clone()) else {
        return;
    };
    let Some(mut hud) = world.get_non_send_mut::<HudView>() else {
        return;
    };
    let Some(front) = hud.draw(&gpu, size, density) else {
        return;
    };
    world.resource_mut::<HudFront>().0 = Some(front);
}

fn ensure_view(world: &mut World, density: f32) -> bool {
    if world.get_non_send::<HudView>().is_some() {
        return true;
    }
    let Some(gpu) = world.get_resource::<CranposeGpu>().map(|gpu| gpu.0.clone()) else {
        return false;
    };
    let factory = Arc::clone(&world.resource::<HudContent>().0);
    let content = factory(world);
    match AppLauncher::new().create_guest_view(&gpu, HUD_FORMAT, 1, 1, density, content) {
        Ok(mut view) => {
            view.shell().renderer().set_transparent_background(true);
            world.insert_non_send(HudView {
                view,
                textures: [None, None],
                front: None,
            });
            true
        }
        Err(error) => {
            error!("Cranpose HUD: {error}");
            false
        }
    }
}

impl HudView {
    // Draws a changed composition into the texture the render world is not
    // reading and returns it as the new front.
    fn draw(&mut self, gpu: &GpuContext, size: UVec2, density: f32) -> Option<HudTexture> {
        if let Err(error) = self.view.resize(size.x, size.y, density) {
            error!("Cranpose HUD resize: {error}");
            return None;
        }
        let back = self.front.map_or(0, |front| 1 - front);
        let fits = |texture: &HudTexture| {
            (texture.texture.width(), texture.texture.height()) == (size.x, size.y)
        };
        if !self.textures[back].as_ref().is_some_and(fits) {
            self.textures[back] = Some(hud_texture(gpu, size));
            self.view.shell().mark_dirty();
        }
        let target = self.textures[back].as_ref()?;
        let view: &wgpu::TextureView = &target.view;
        match self.view.draw(&target.texture, view) {
            Ok(true) => {
                self.front = Some(back);
                Some(target.clone())
            }
            Ok(false) => None,
            Err(error) => {
                error!("Cranpose HUD draw: {error}");
                None
            }
        }
    }
}

fn hud_texture(gpu: &GpuContext, size: UVec2) -> HudTexture {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Cranpose HUD"),
        size: wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: HUD_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    HudTexture {
        texture,
        view: view.into(),
    }
}

fn init_hud_pipeline(
    mut commands: Commands,
    fullscreen: Res<FullscreenShader>,
    assets: Res<AssetServer>,
) {
    commands.insert_resource(HudPipeline {
        layout: BindGroupLayoutDescriptor::new(
            "cranpose_hud",
            &BindGroupLayoutEntries::single(
                ShaderStages::FRAGMENT,
                texture_2d(TextureSampleType::Float { filterable: false }),
            ),
        ),
        fullscreen: fullscreen.clone(),
        shader: load_embedded_asset!(assets.as_ref(), "hud.wgsl"),
    });
}

impl SpecializedRenderPipeline for HudPipeline {
    type Key = TextureFormat;

    fn specialize(&self, format: TextureFormat) -> RenderPipelineDescriptor {
        let entry_point = if format.is_srgb() {
            "over_linear"
        } else {
            "over_encoded"
        };
        RenderPipelineDescriptor {
            label: Some("cranpose_hud".into()),
            layout: vec![self.layout.clone()],
            vertex: self.fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                entry_point: Some(entry_point.into()),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        }
    }
}

fn prepare_hud_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<HudPipeline>>,
    hud: Res<HudPipeline>,
    views: Query<(Entity, &ViewTarget, Option<&ViewHudPipeline>), With<HudCamera>>,
) {
    for (entity, target, prepared) in &views {
        let Some(format) = target.out_texture_view_format() else {
            continue;
        };
        if prepared.is_none_or(|prepared| prepared.format != format) {
            let id = pipelines.specialize(&pipeline_cache, &hud, format);
            commands
                .entity(entity)
                .insert(ViewHudPipeline { id, format });
        }
    }
}

fn hud_pass(
    view: ViewQuery<(&ViewTarget, &ViewHudPipeline)>,
    front: Option<Res<HudFront>>,
    pipeline_cache: Res<PipelineCache>,
    hud: Res<HudPipeline>,
    mut bind_group: Local<Option<(TextureViewId, BindGroup)>>,
    mut ctx: RenderContext,
) {
    let (target, prepared) = view.into_inner();
    let Some(texture) = front.as_ref().and_then(|front| front.0.as_ref()) else {
        return;
    };
    let Some(pipeline) = pipeline_cache.get_render_pipeline(prepared.id) else {
        return;
    };
    let bind_group = match &mut *bind_group {
        Some((id, bind_group)) if *id == texture.view.id() => bind_group,
        cached => {
            let created = ctx.render_device().create_bind_group(
                "cranpose_hud",
                &pipeline_cache.get_bind_group_layout(&hud.layout),
                &BindGroupEntries::single(&texture.view),
            );
            &cached.insert((texture.view.id(), created)).1
        }
    };
    let Some(attachment) = target.out_texture_color_attachment(None) else {
        return;
    };
    let mut pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("cranpose_hud"),
            color_attachments: &[Some(attachment)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.draw(0..3, 0..1);
}
