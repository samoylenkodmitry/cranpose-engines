//! Cranpose compositions on meshes in a Bevy world, such as the screen of a
//! computer in a game, which the pointer and fingers use.

use std::sync::Arc;

use bevy::{
    asset::embedded_asset,
    pbr::{Material, MaterialPlugin},
    prelude::*,
    reflect::TypePath,
    render::{
        Render, RenderApp, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, Sampler},
        renderer::RenderDevice,
        texture::GpuImage,
    },
    shader::ShaderRef,
};
use cranpose::{AppLauncher, GuestView};

use crate::{ContentFactory, gpu::CranposeGpu};

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Draws the composition of every [`CranposeScreen`] on its entity's mesh,
/// and sends it the pointer, the fingers and the keys that land there.
pub struct CranposeScreenPlugin;

/// A Cranpose composition on this entity's mesh.
///
/// Give the entity a `Mesh3d` with UVs, such as a `Rectangle`, that keeps
/// its CPU copy (the default) so the pointer can find it; the plugin gives
/// it the material that shows the composition. The composition is `size` dp
/// and is drawn at `density` pixels per dp. The mesh's UVs place it: (0, 0)
/// is its top left corner and (1, 1) its bottom right.
///
/// A press goes to the screen nearest the camera under the pointer, after
/// the HUD, when the screen is within its reach of the camera;
/// [`crate::CranposeInput`] tells the game what the screens took.
/// Meshes without a CPU copy, such as ones made with
/// `RenderAssetUsages::RENDER_WORLD`, do not hide a screen from the pointer.
/// Keys go to the screen pressed last, after the HUD.
#[derive(Component, Clone)]
pub struct CranposeScreen {
    /// The composition's width and height in dp.
    pub size: Vec2,
    /// Pixels per dp of the texture the composition draws into.
    pub density: f32,
    /// How far from the camera, in world units, the pointer reaches the
    /// screen. Unlimited unless set with [`Self::with_reach`].
    pub reach: f32,
    content: Arc<ContentFactory>,
}

impl CranposeScreen {
    /// A screen of `size` dp at `density` pixels per dp. `content` gets
    /// Bevy's world once, to share state with the game, and returns the
    /// composition's root.
    pub fn new(
        size: Vec2,
        density: f32,
        content: impl Fn(&mut World) -> Box<dyn FnMut()> + Send + Sync + 'static,
    ) -> Self {
        Self {
            size,
            density,
            reach: f32::INFINITY,
            content: Arc::new(content),
        }
    }

    /// The pointer reaches the screen only from `reach` world units or
    /// nearer, such as a computer a player walks up to before using it.
    pub fn with_reach(mut self, reach: f32) -> Self {
        self.reach = reach;
        self
    }

    fn pixels(&self) -> UVec2 {
        (self.size * self.density)
            .round()
            .as_uvec2()
            .max(UVec2::ONE)
    }
}

// Shows a screen's texture, which Cranpose fills with sRGB-encoded colors,
// lit by nothing: a screen gives off its own light.
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
struct ScreenMaterial {
    #[texture(0)]
    #[sampler(1)]
    screen: Handle<Image>,
}

impl Material for ScreenMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://cranpose_bevy/screen.wesl".into()
    }
}

/// The compositions of the screens, on Bevy's main thread.
#[derive(Default)]
pub(crate) struct ScreenViews {
    screens: Vec<ScreenView>,
}

struct ScreenView {
    entity: Entity,
    view: GuestView,
    pixels: UVec2,
    density: f32,
    texture: wgpu::Texture,
    target: wgpu::TextureView,
    image: Handle<Image>,
    material: Handle<ScreenMaterial>,
}

// The screens' textures, for the render world to give their images.
#[derive(Resource, Clone, Default, ExtractResource)]
#[extract_app(RenderApp)]
struct ScreenTextures(Vec<ScreenTexture>);

#[derive(Clone)]
struct ScreenTexture {
    image: AssetId<Image>,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Plugin for CranposeScreenPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "screen.wesl");
        crate::input::add_to(app);
        app.insert_non_send(ScreenViews::default())
            .init_resource::<ScreenTextures>()
            .add_plugins((
                MaterialPlugin::<ScreenMaterial>::default(),
                ExtractResourcePlugin::<ScreenTextures>::default(),
            ))
            .add_systems(Last, update_screens);
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            // After the extracted textures arrive, before the materials
            // look up their images.
            render.add_systems(
                Render,
                register_textures
                    .after(RenderSystems::ExtractCommands)
                    .before(RenderSystems::PrepareAssets),
            );
        }
    }

    fn finish(&self, app: &mut App) {
        crate::gpu::insert(app);
    }
}

impl ScreenViews {
    pub(crate) fn view(&mut self, entity: Entity) -> Option<&mut GuestView> {
        self.screens
            .iter_mut()
            .find(|screen| screen.entity == entity)
            .map(|screen| &mut screen.view)
    }
}

// Starts the compositions of new screens, sizes the ones whose size changed,
// drops the ones whose entity went, and draws the ones that changed.
fn update_screens(world: &mut World, screens: &mut QueryState<(Entity, &CranposeScreen)>) {
    if !world.contains_resource::<CranposeGpu>() {
        return;
    }
    let mut changed = false;
    let mut pending = Vec::new();
    if let Some(mut views) = world.remove_non_send::<ScreenViews>() {
        let before = views.screens.len();
        views
            .screens
            .retain(|view| screens.get(world, view.entity).is_ok());
        changed |= views.screens.len() != before;
        for (entity, screen) in screens.iter(world) {
            let current = views.screens.iter().find(|view| view.entity == entity);
            if current
                .is_none_or(|view| view.pixels != screen.pixels() || view.density != screen.density)
            {
                pending.push((entity, screen.clone()));
            }
        }
        for (entity, screen) in pending {
            changed |= place(world, &mut views, entity, &screen);
        }
        for screen in &mut views.screens {
            if let Err(error) = screen.view.draw(&screen.texture, &screen.target) {
                error!("Cranpose screen draw: {error}");
            }
        }
        if changed {
            let textures = views
                .screens
                .iter()
                .map(|screen| ScreenTexture {
                    image: screen.image.id(),
                    texture: screen.texture.clone(),
                    view: screen.target.clone(),
                })
                .collect();
            world.resource_mut::<ScreenTextures>().0 = textures;
        }
        world.insert_non_send(views);
    }
}

// Gives a screen a composition and a texture of its size; returns whether
// it did.
fn place(
    world: &mut World,
    views: &mut ScreenViews,
    entity: Entity,
    screen: &CranposeScreen,
) -> bool {
    let gpu = world.resource::<CranposeGpu>().0.clone();
    let pixels = screen.pixels();
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Cranpose screen"),
        size: wgpu::Extent3d {
            width: pixels.x,
            height: pixels.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target = texture.create_view(&wgpu::TextureViewDescriptor::default());
    if let Some(view) = views.screens.iter_mut().find(|view| view.entity == entity) {
        if let Err(error) = view.view.resize(pixels.x, pixels.y, screen.density) {
            error!("Cranpose screen resize: {error}");
            return false;
        }
        view.view.shell().mark_dirty();
        view.pixels = pixels;
        view.density = screen.density;
        view.texture = texture;
        view.target = target;
        // A changed material builds its bind group again, with the new
        // texture.
        world
            .resource_mut::<Assets<ScreenMaterial>>()
            .get_mut(&view.material);
        return true;
    }
    let content = (screen.content)(world);
    let view = match AppLauncher::new().create_guest_view(
        &gpu,
        FORMAT,
        pixels.x,
        pixels.y,
        screen.density,
        content,
    ) {
        Ok(view) => view,
        Err(error) => {
            error!("Cranpose screen: {error}");
            return false;
        }
    };
    let image = world.resource::<Assets<Image>>().reserve_handle();
    let material = world
        .resource_mut::<Assets<ScreenMaterial>>()
        .add(ScreenMaterial {
            screen: image.clone(),
        });
    world
        .entity_mut(entity)
        .insert(MeshMaterial3d(material.clone()));
    views.screens.push(ScreenView {
        entity,
        view,
        pixels,
        density: screen.density,
        texture,
        target,
        image,
        material,
    });
    true
}

// Makes each screen texture the image its material samples. The images have
// no asset in the main world, so nothing else adds or removes them.
fn register_textures(
    textures: Res<ScreenTextures>,
    mut images: ResMut<RenderAssets<GpuImage>>,
    device: Res<RenderDevice>,
    mut sampler: Local<Option<Sampler>>,
    mut registered: Local<Vec<AssetId<Image>>>,
) {
    if !textures.is_changed() {
        return;
    }
    let sampler = sampler
        .get_or_insert_with(|| {
            device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Cranpose screen"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..default()
            })
        })
        .clone();
    for gone in registered.iter() {
        if textures.0.iter().all(|texture| texture.image != *gone) {
            images.remove(*gone);
        }
    }
    registered.clear();
    for screen in &textures.0 {
        registered.push(screen.image);
        if images
            .get(screen.image)
            .is_some_and(|image| *image.texture == screen.texture)
        {
            continue;
        }
        images.insert(
            screen.image,
            GpuImage {
                texture: screen.texture.clone().into(),
                texture_view: screen.view.clone().into(),
                sampler: sampler.clone(),
                texture_descriptor: wgpu::TextureDescriptor {
                    label: Some("Cranpose screen"),
                    size: screen.texture.size(),
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: FORMAT,
                    usage: screen.texture.usage(),
                    view_formats: &[],
                },
                texture_view_descriptor: None,
                had_data: true,
            },
        );
    }
}
