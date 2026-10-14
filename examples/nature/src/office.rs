//! Office things left on the river's bank, as in the backrooms: an old desk
//! with a tube monitor, the computer on its side by the desk, a fallen
//! chair, and a piece of yellow wall with a doorway and a tube light. Every
//! engine builds the same parts from [`Office::place`].

use crate::{Terrain, Walker};

/// How near the camera must be for the pointer to reach the screen, in
/// metres.
pub const REACH: f32 = 3.0;
/// The size of the screen's composition in dp: a VGA screen.
pub const SCREEN_DP: [f32; 2] = [320.0, 240.0];
/// Pixels per dp of the screen's composition.
pub const SCREEN_DENSITY: f32 = 2.0;

const DESK_HEIGHT: f32 = 0.74;
// The picture tube's glass: width, height, and how far it bulges in the
// middle, in metres.
const GLASS: [f32; 3] = [0.31, 0.235, 0.014];

/// A part's shape, centred on its place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// A box of this width, height and depth.
    Cuboid([f32; 3]),
    /// An upright cylinder.
    Cylinder { radius: f32, height: f32 },
    /// The monitor's glass, [`glass_mesh`], whose front faces +z.
    Glass,
}

/// What a part is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    Plastic,
    Desk,
    Fabric,
    Wallpaper,
    Carpet,
    /// The tube light, which flickers: see [`lamp_lit`].
    Lamp,
    /// The monitor's power light.
    Led,
    /// The monitor's glass, which shows the screen's composition.
    Screen,
}

impl Look {
    /// The surface's colour, in sRGB.
    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Plastic => [0.76, 0.72, 0.6],
            Self::Desk => [0.6, 0.56, 0.48],
            Self::Fabric => [0.18, 0.2, 0.26],
            Self::Wallpaper => [0.8, 0.72, 0.42],
            Self::Carpet => [0.52, 0.47, 0.3],
            Self::Lamp | Self::Led | Self::Screen => [1.0, 1.0, 1.0],
        }
    }

    /// The surface's roughness.
    pub fn roughness(self) -> f32 {
        match self {
            Self::Plastic => 0.7,
            Self::Desk => 0.8,
            Self::Fabric | Self::Wallpaper => 0.95,
            Self::Carpet => 1.0,
            Self::Lamp | Self::Led | Self::Screen => 0.5,
        }
    }

    /// The light a glowing part gives off, in linear RGB, relative to the
    /// tube light's.
    pub fn glow(self) -> Option<[f32; 3]> {
        match self {
            Self::Lamp => Some([1.0, 1.0, 0.85]),
            Self::Led => Some([0.0, 0.75, 0.0]),
            _ => None,
        }
    }
}

/// One part of the office, placed in the world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub shape: Shape,
    pub look: Look,
    pub translation: [f32; 3],
    /// A unit quaternion: x, y, z, w.
    pub rotation: [f32; 4],
}

/// The office's parts.
#[derive(Clone, Debug, PartialEq)]
pub struct Office {
    pub parts: Vec<Part>,
}

impl Office {
    /// The office on `terrain`'s river bank below the start: the walker
    /// starts on a slope eleven metres over the river, looking along +x, and
    /// the bank comes into view from about 34 metres on. The desk's legs
    /// reach down to the ground under each of them, so it stands level.
    pub fn place(terrain: &Terrain) -> Self {
        let start = Walker::start(terrain).position;
        let desk = terrain.bank(start[0] + 38.0, start[2], 0.35);
        let facing = turn_y((start[0] - desk[0]).atan2(start[2] - desk[2]));
        let mut parts = Vec::new();
        place_desk(&mut parts, terrain, desk, facing);
        let wall = terrain.bank(desk[0] + 7.0, start[2], 0.6);
        place_wall(&mut parts, [wall[0], wall[1] - 0.15, wall[2]], facing);
        Self { parts }
    }

    /// The monitor's glass.
    pub fn screen(&self) -> Option<&Part> {
        self.parts.iter().find(|part| part.look == Look::Screen)
    }
}

/// Where a ray from `origin` along `direction` meets the front of the
/// screen `part`: its distance, in units of `direction`, and the point in
/// the screen's composition, in dp. The glass counts as flat.
pub fn screen_hit(part: &Part, origin: [f32; 3], direction: [f32; 3]) -> Option<(f32, [f32; 2])> {
    let back = inverse(part.rotation);
    let from = rotate(back, sub(origin, part.translation));
    let along = rotate(back, direction);
    if along[2] >= 0.0 {
        return None;
    }
    let distance = -from[2] / along[2];
    if distance < 0.0 {
        return None;
    }
    let u = (from[0] + along[0] * distance) / GLASS[0] + 0.5;
    let v = 0.5 - (from[1] + along[1] * distance) / GLASS[1];
    ((0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v))
        .then_some((distance, [u * SCREEN_DP[0], v * SCREEN_DP[1]]))
}

/// Whether the tube light shines `seconds` after the scene started: it goes
/// out for a moment every few seconds.
pub fn lamp_lit(seconds: f32) -> bool {
    !(seconds % 4.3 > 4.1 || seconds % 7.9 > 7.83)
}

/// The monitor's glass: a picture tube's front, bulging towards +z in the
/// middle. Its normals face +z, and its UVs run from (0, 0) at the top left
/// to (1, 1) at the bottom right.
#[derive(Clone, Debug, PartialEq)]
pub struct GlassMesh {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

/// The [`Shape::Glass`] mesh.
pub fn glass_mesh() -> GlassMesh {
    const COLUMNS: u32 = 16;
    const ROWS: u32 = 12;
    let [width, height, bulge] = GLASS;
    let points = ((COLUMNS + 1) * (ROWS + 1)) as usize;
    let mut positions = Vec::with_capacity(points);
    let mut uvs = Vec::with_capacity(points);
    for row in 0..=ROWS {
        for column in 0..=COLUMNS {
            let (u, v) = (column as f32 / COLUMNS as f32, row as f32 / ROWS as f32);
            let rise = (1.0 - (2.0 * u - 1.0).powi(2)) * (1.0 - (2.0 * v - 1.0).powi(2));
            positions.push([(u - 0.5) * width, (0.5 - v) * height, bulge * rise]);
            uvs.push([u, v]);
        }
    }
    let mut indices = Vec::with_capacity((COLUMNS * ROWS * 6) as usize);
    for row in 0..ROWS {
        for column in 0..COLUMNS {
            let top_left = row * (COLUMNS + 1) + column;
            let bottom_left = top_left + COLUMNS + 1;
            indices.extend([
                top_left,
                bottom_left,
                top_left + 1,
                top_left + 1,
                bottom_left,
                bottom_left + 1,
            ]);
        }
    }
    GlassMesh {
        positions,
        uvs,
        indices,
    }
}

// A place to put parts relative to.
#[derive(Clone, Copy)]
struct Place {
    at: [f32; 3],
    rotation: [f32; 4],
}

impl Place {
    fn then(self, at: [f32; 3], rotation: [f32; 4]) -> Self {
        Self {
            at: add(self.at, rotate(self.rotation, at)),
            rotation: multiply(self.rotation, rotation),
        }
    }

    fn part(self, parts: &mut Vec<Part>, shape: Shape, look: Look, at: [f32; 3]) {
        let place = self.then(at, IDENTITY);
        parts.push(Part {
            shape,
            look,
            translation: place.at,
            rotation: place.rotation,
        });
    }
}

fn place_desk(parts: &mut Vec<Part>, terrain: &Terrain, at: [f32; 3], facing: [f32; 4]) {
    let corners = [(-0.55, -0.25), (0.55, -0.25), (-0.55, 0.25), (0.55, 0.25)];
    let ground_under = |x: f32, z: f32| {
        let world = add(at, rotate(facing, [x, 0.0, z]));
        terrain.height(world[0], world[2])
    };
    let top = corners
        .iter()
        .map(|&(x, z)| ground_under(x, z))
        .fold(f32::MIN, f32::max)
        + DESK_HEIGHT;
    let desk = Place {
        at: [at[0], top, at[2]],
        rotation: facing,
    };
    desk.part(
        parts,
        Shape::Cuboid([1.2, 0.04, 0.6]),
        Look::Desk,
        [0.0, 0.02, 0.0],
    );
    for (x, z) in corners {
        let length = top - ground_under(x, z) + 0.05;
        let leg = Shape::Cuboid([0.04, length, 0.04]);
        desk.part(parts, leg, Look::Desk, [x, -length / 2.0, z]);
    }
    desk.then([0.02, 0.055, 0.19], turn_y(-0.06)).part(
        parts,
        Shape::Cuboid([0.44, 0.03, 0.15]),
        Look::Plastic,
        [0.0; 3],
    );
    place_monitor(parts, desk.then([0.0, 0.04, -0.06], turn_y(0.1)));
    // The computer lies on its side by the desk, and the chair fell over.
    let tower = add(at, rotate(facing, [0.85, 0.0, 0.05]));
    Place {
        at: [
            tower[0],
            terrain.height(tower[0], tower[2]) + 0.08,
            tower[2],
        ],
        rotation: multiply(facing, turn_z(1.52)),
    }
    .part(
        parts,
        Shape::Cuboid([0.19, 0.42, 0.44]),
        Look::Plastic,
        [0.0; 3],
    );
    let chair = add(at, rotate(facing, [-0.95, 0.0, 0.7]));
    let chair = Place {
        at: [
            chair[0],
            terrain.height(chair[0], chair[2]) + 0.24,
            chair[2],
        ],
        rotation: multiply(multiply(facing, turn_y(0.7)), turn_z(1.4)),
    };
    chair.part(
        parts,
        Shape::Cuboid([0.46, 0.07, 0.46]),
        Look::Fabric,
        [0.0; 3],
    );
    let back = Shape::Cuboid([0.44, 0.52, 0.06]);
    chair.part(parts, back, Look::Fabric, [0.0, 0.3, -0.21]);
    let post = Shape::Cylinder {
        radius: 0.03,
        height: 0.38,
    };
    chair.part(parts, post, Look::Desk, [0.0, -0.22, 0.0]);
}

// A fourteen-inch tube monitor, its front along +z, standing on its foot.
fn place_monitor(parts: &mut Vec<Part>, monitor: Place) {
    let case = [
        ([0.24, 0.03, 0.22], [0.0, 0.015, 0.0]),
        ([0.1, 0.05, 0.1], [0.0, 0.05, 0.0]),
        ([0.4, 0.36, 0.3], [0.0, 0.25, 0.0]),
        ([0.3, 0.28, 0.16], [0.0, 0.25, -0.22]),
    ];
    for (size, at) in case {
        monitor.part(parts, Shape::Cuboid(size), Look::Plastic, at);
    }
    let led = Shape::Cuboid([0.012, 0.012, 0.004]);
    monitor.part(parts, led, Look::Led, [0.16, 0.1, 0.151]);
    monitor.part(parts, Shape::Glass, Look::Screen, [0.0, 0.265, 0.151]);
}

// A piece of yellow wall with a doorway, standing on a patch of carpet, and
// a tube light above the door.
fn place_wall(parts: &mut Vec<Part>, at: [f32; 3], facing: [f32; 4]) {
    let wall = Place {
        at,
        rotation: facing,
    };
    let pieces = [
        ([1.2, 2.8, 0.12], [-1.0, 1.4, 0.0]),
        ([1.2, 2.8, 0.12], [1.0, 1.4, 0.0]),
        ([0.8, 0.6, 0.12], [0.0, 2.5, 0.0]),
    ];
    for (size, at) in pieces {
        wall.part(parts, Shape::Cuboid(size), Look::Wallpaper, at);
    }
    let carpet = Shape::Cuboid([3.8, 0.03, 2.6]);
    wall.part(parts, carpet, Look::Carpet, [0.0, 0.16, 1.2]);
    let lamp = Shape::Cuboid([0.9, 0.05, 0.14]);
    wall.part(parts, lamp, Look::Lamp, [0.0, 2.68, 0.13]);
}

const IDENTITY: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn turn_y(angle: f32) -> [f32; 4] {
    let (sin, cos) = (angle / 2.0).sin_cos();
    [0.0, sin, 0.0, cos]
}

fn turn_z(angle: f32) -> [f32; 4] {
    let (sin, cos) = (angle / 2.0).sin_cos();
    [0.0, 0.0, sin, cos]
}

fn multiply(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

fn inverse([x, y, z, w]: [f32; 4]) -> [f32; 4] {
    [-x, -y, -z, w]
}

fn rotate([x, y, z, w]: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let axis = [x, y, z];
    let twice = cross(axis, v).map(|c| 2.0 * c);
    add(add(v, twice.map(|c| w * c)), cross(axis, twice))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
