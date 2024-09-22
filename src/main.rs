//! This example demonstrates how to create a custom mesh,
//! assign a custom UV mapping for a custom texture,
//! and how to change the UV mapping at run-time.

use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use rand::prelude::*;
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::render::{
    mesh::{Indices, VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with With, they're usually not queried directly since they don't contain information within them.
#[derive(Component)]
struct Mountain {}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct MountainMaterial {}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MaterialPlugin::<MountainMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, input_handler)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<MountainMaterial>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    // Create and save a handle to the mesh.
    let cube_mesh_handle: Handle<Mesh> = meshes.add(create_mountain_mesh());

    commands.spawn((
        MaterialMeshBundle {
            mesh: cube_mesh_handle,
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            //material: materials.add(MountainMaterial {}),
            material: std_materials.add(StandardMaterial {
                metallic: 1.0,
                base_color: Color::srgb(1.0, 0.5, 0.5),
                ..default()
            }),
            ..default()
        },
        Mountain {},
    ));

    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    let camera_and_light_transform =
        Transform::from_xyz(1.8, 1.8, 1.8).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera in 3D space.
    commands.spawn(Camera3dBundle {
        transform: camera_and_light_transform,
        ..default()
    });

    // Light up the scene.
    commands.spawn(PointLightBundle {
        transform: camera_and_light_transform,
        ..default()
    });

    // Text to describe the controls.
    commands.spawn(
        TextBundle::from_section(
            "Controls:\nMouse to look around\nWASD: movement",
            TextStyle::default(),
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );

    let mut primary_window = q_windows.single_mut();
    // for a game that doesn't use the cursor (like a shooter):
    // use `Locked` mode to keep the cursor in one place
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;

    // also hide the cursor
    primary_window.cursor.visible = false;
}

// System to receive input from the user,
// check out examples/input/ for more examples about user input.
fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mesh_query: Query<&Handle<Mesh>, With<Mountain>>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
    mut evr_motion: EventReader<MouseMotion>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Mountain>)>,
) {
    let mut camera = camera.get_single_mut().unwrap();
    for ev in evr_motion.read() {
        // rotate the camera relative to the x and y
        camera.rotate_local(
            Quat::from_rotation_x(-ev.delta.y / 1000.0)
                * Quat::from_rotation_y(-ev.delta.x / 1000.0),
        );
    }

    if keyboard_input.pressed(KeyCode::KeyW) {
        let forward = camera.forward();
        camera.translation += forward * 0.01;
    }

    if keyboard_input.pressed(KeyCode::KeyA) {
        let left = camera.left();
        camera.translation += left * 0.01;
    }

    if keyboard_input.pressed(KeyCode::KeyS) {
        let back = camera.back();
        camera.translation += back * 0.01;
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        let right = camera.right();
        camera.translation += right*0.01;
    }
}

// smoothly interpolates between some points using a special polynomial from this video's beginning:
// https://www.youtube.com/watch?v=BFld4EBO2RE
// interpolate_step is how many points there are between random points
fn interpolate_random_points(
    points: &[Vec<f32>],
    xi: usize,
    yi: usize,
    interpolate_step: usize,
) -> f32 {
    let s_polynomial = |val: f32| 3.0 * (val * val * val) - 2.0 * val * val;
    let rand_a = points[xi / interpolate_step][yi / interpolate_step];
    let rand_b = points[xi / interpolate_step + 1][yi / interpolate_step];
    let rand_c = points[xi / interpolate_step][yi / interpolate_step + 1];
    let rand_d = points[xi / interpolate_step + 1][yi / interpolate_step + 1];

    let rel_x =
        ((xi - (interpolate_step * (xi / interpolate_step))) as f32) / (interpolate_step as f32);
    let rel_y =
        ((yi - (interpolate_step * (yi / interpolate_step))) as f32) / (interpolate_step as f32);
    // interpolate smoothly between them
    rand_a
        + (rand_b - rand_a) * s_polynomial(rel_x)
        + (rand_c - rand_a) * s_polynomial(rel_y)
        + (rand_a - rand_b - rand_c + rand_d) * s_polynomial(rel_x) * s_polynomial(rel_y)
}

fn create_mountain_mesh() -> Mesh {
    let mut random_positions: Vec<Vec<f32>> = vec![];
    let mut rng = rand::thread_rng();
    // add a ton of random positions so we never go out of bounds
    for _i in 0..1000 {
        let mut random_row = vec![];
        for _j in 0..1000 {
            random_row.push(rng.gen());
        }
        random_positions.push(random_row);
    }

    let mut vertex_positions = vec![];
    let mut uv_positions = vec![];
    let mut triangles = vec![];
    let mut normals = vec![];

    let x_max = 200;
    let interpolate_step = 20;
    let y_max = 200;
    let last_index = (x_max * y_max) - 1;

    for xi in 0..x_max {
        for yi in 0..y_max {
            let z = interpolate_random_points(&random_positions, xi, yi, interpolate_step)
                + 0.5
                    * interpolate_random_points(
                        &random_positions,
                        xi * 2,
                        yi * 2,
                        interpolate_step,
                    );

            vertex_positions.push([
                ((xi as f32) / (x_max as f32)) * 4.0 - 1.0,
                ((yi as f32) / (y_max as f32)) * 4.0 - 1.0,
                z,
            ]);
            uv_positions.push([(xi as f32) / (x_max as f32), (yi as f32) / (y_max as f32)]);
            normals.push([0.0, 0.0, 1.0]);

            // we make squares, so two triangles per index
            let index = xi * y_max + yi;
            let index_right = (xi + 1) * y_max + yi;
            let index_down = xi * y_max + yi + 1;
            let index_down_right = (xi + 1) * y_max + yi + 1;

            if index_down_right <= last_index {
                triangles.extend(vec![
                    index as u32,
                    index_down_right as u32,
                    index_right as u32,
                ]);
                triangles.extend(vec![
                    index as u32,
                    index_down as u32,
                    index_down_right as u32,
                ]);
            }
        }
    }

    // Keep the mesh data accessible in future frames to be able to mutate it in toggle_texture.
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        // Each array is an [x, y, z] coordinate in local space.
        // The camera coordinate space is right-handed x-right, y-up, z-back. This means "forward" is -Z.
        // Meshes always rotate around their local [0, 0, 0] when a rotation is applied to their Transform.
        // By centering our mesh around the origin, rotating the mesh preserves its center of mass.
        vertex_positions,
    )
    // make uv the same as vertex positions XD
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv_positions)
    //.with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(triangles))
    .with_computed_normals()
}

// Function that changes the UV mapping of the mesh, to apply the other texture.
fn toggle_texture(mesh_to_change: &mut Mesh) {}

impl Material for MountainMaterial {
    fn fragment_shader() -> ShaderRef {
        "animate_shader.wgsl".into()
    }
}
