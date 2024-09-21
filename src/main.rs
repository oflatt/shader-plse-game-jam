//! This example demonstrates how to create a custom mesh,
//! assign a custom UV mapping for a custom texture,
//! and how to change the UV mapping at run-time.

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
) {
    // Create and save a handle to the mesh.
    let cube_mesh_handle: Handle<Mesh> = meshes.add(create_cube_mesh());

    commands.spawn((
        MaterialMeshBundle {
            mesh: cube_mesh_handle,
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            material: materials.add(MountainMaterial {}),
            //material: std_materials.add(StandardMaterial { ..default() }),
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
            "Controls:\nSpace: Change UVs\nX/Y/Z: Rotate\nR: Reset orientation",
            TextStyle::default(),
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

// System to receive input from the user,
// check out examples/input/ for more examples about user input.
fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mesh_query: Query<&Handle<Mesh>, With<Mountain>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&mut Transform, With<Mountain>>,
    time: Res<Time>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let mesh_handle = mesh_query.get_single().expect("Query not successful");
        let mesh = meshes.get_mut(mesh_handle).unwrap();
        toggle_texture(mesh);
    }
    if keyboard_input.pressed(KeyCode::KeyX) {
        for mut transform in &mut query {
            transform.rotate_x(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyY) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyZ) {
        for mut transform in &mut query {
            transform.rotate_z(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyR) {
        for mut transform in &mut query {
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
        }
    }
}

fn create_cube_mesh() -> Mesh {
    let mut vertex_positions = vec![];
    let mut uv_positions = vec![];
    let mut triangles = vec![];
    let mut normals = vec![];

    let x_max = 100;
    let y_max = 100;
    let last_index = (x_max * y_max) - 1;
    for xi in 0..x_max {
        for yi in 0..y_max {
            vertex_positions.push([
                (xi as f32) / (x_max as f32),
                (yi as f32) / (y_max as f32),
                (xi as f32) / (x_max as f32) + (yi as f32) / (y_max as f32),
            ]);
            normals.push([0.0, 0.0, 1.0]);

            // we make squares, so two triangles per index
            let index = xi * y_max + yi;
            let index_right = (xi + 1) * y_max + yi;
            let index_down = xi * y_max + yi + 1;
            let index_down_right = (xi + 1) * y_max + yi + 1;

            if index_down_right <= last_index {
                triangles.extend(vec![index, index_down_right, index_right]);
                triangles.extend(vec![index, index_down, index_down_right]);
            }
        }
    }

    for pos in vertex_positions.iter() {
        uv_positions.push([pos[0], pos[1]])
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
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(triangles))
}

// Function that changes the UV mapping of the mesh, to apply the other texture.
fn toggle_texture(mesh_to_change: &mut Mesh) {
    // Get a mutable reference to the values of the UV attribute, so we can iterate over it.
    let uv_attribute = mesh_to_change.attribute_mut(Mesh::ATTRIBUTE_UV_0).unwrap();
    // The format of the UV coordinates should be Float32x2.
    let VertexAttributeValues::Float32x2(uv_attribute) = uv_attribute else {
        panic!("Unexpected vertex format, expected Float32x2.");
    };

    // Iterate over the UV coordinates, and change them as we want.
    for uv_coord in uv_attribute.iter_mut() {
        // If the UV coordinate points to the upper, "dirt+grass" part of the texture...
        if (uv_coord[1] + 0.5) < 1.0 {
            // ... point to the equivalent lower, "sand+water" part instead,
            uv_coord[1] += 0.5;
        } else {
            // else, point back to the upper, "dirt+grass" part.
            uv_coord[1] -= 0.5;
        }
    }
}

impl Material for MountainMaterial {
    fn fragment_shader() -> ShaderRef {
        "animate_shader.wgsl".into()
    }
}
