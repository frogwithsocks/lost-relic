// https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/tiled/tiled.rs

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::render::render_resource::TextureUsages;
use bevy_ecs_tilemap::prelude::*;
use std::{collections::HashMap, io::BufReader};

use bevy::asset::{AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset};
use bevy::reflect::TypeUuid;

use crate::camera::CameraAnchor;
use crate::collide::{Collider, ColliderKind};
use crate::map::{CellTower, BLOCK_SIZE};
use crate::player::{PlayerBundle, PlayerTexture};
use crate::velocity::{Velocity, Gravity};

#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<TiledMap>()
            .add_asset_loader(TiledLoader)
            .add_system(process_loaded_tile_maps.label("map_update"))
            .add_system(set_texture_filters_to_nearest);
    }
}

#[derive(Bundle)]
pub struct BoxBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
    velocity: Velocity,
    gravity: Gravity,
}

#[derive(TypeUuid)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct TiledMap {
    pub map: tiled::Map,
    pub tilesets: HashMap<usize, Handle<Image>>,
}

#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMap>,
    pub map: Map,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub struct TiledLoader;

impl AssetLoader for TiledLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut loader = tiled::Loader::new();
            let map = loader.load_tmx_map_from(BufReader::new(bytes), load_context.path())?;

            let mut dependencies = Vec::new();
            let mut handles = HashMap::default();

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                let tile_path = tileset.image.as_ref().unwrap().source.clone();
                let asset_path = AssetPath::new(tile_path, None);
                let texture: Handle<Image> = load_context.get_handle(asset_path.clone());

                handles.insert(tileset_index, texture.clone());

                dependencies.push(asset_path);
            }

            let loaded_asset = LoadedAsset::new(TiledMap {
                map,
                tilesets: handles,
            });
            load_context.set_default_asset(loaded_asset.with_dependencies(dependencies));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

pub fn process_loaded_tile_maps(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    maps: Res<Assets<TiledMap>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &Handle<TiledMap>, &mut Map)>,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
    layer_query: Query<&Layer>,
    chunk_query: Query<&Chunk>,
    player_texture_res: Res<PlayerTexture>,
) {
    let mut changed_maps = Vec::<Handle<TiledMap>>::default();
    for event in map_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                changed_maps.push(handle.clone());
            }
            AssetEvent::Modified { handle } => {
                changed_maps.push(handle.clone());
            }
            AssetEvent::Removed { handle } => {
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps = changed_maps
                    .into_iter()
                    .filter(|changed_handle| changed_handle == handle)
                    .collect();
            }
        }
    }

    // If we have new map entities add them to the changed_maps list.
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.clone_weak());
    }

    for changed_map in changed_maps.iter() {
        for (_, map_handle, mut map) in query.iter_mut() {
            // only deal with currently changed map
            if map_handle != changed_map {
                continue;
            }
            if let Some(tiled_map) = maps.get(map_handle) {
                // Despawn all tiles/chunks/layers.
                for (layer_id, layer_entity) in map.get_layers() {
                    if let Ok(layer) = layer_query.get(layer_entity) {
                        for x in 0..layer.get_layer_size_in_tiles().0 {
                            for y in 0..layer.get_layer_size_in_tiles().1 {
                                let tile_pos = TilePos(x, y);
                                let chunk_pos = ChunkPos(
                                    tile_pos.0 / layer.settings.chunk_size.0,
                                    tile_pos.1 / layer.settings.chunk_size.1,
                                );
                                if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                                    if let Ok(chunk) = chunk_query.get(chunk_entity) {
                                        let chunk_tile_pos = chunk.to_chunk_pos(tile_pos);

                                        if let Ok(chunk_tile_pos) = chunk_tile_pos {
                                            if let Some(tile) =
                                                chunk.get_tile_entity(chunk_tile_pos)
                                            {
                                                commands.entity(tile).despawn_recursive();
                                            }
                                        }
                                    }

                                    commands.entity(chunk_entity).despawn_recursive();
                                }
                            }
                        }
                    }
                    map.remove_layer(&mut commands, layer_id);
                }
                let mut first_gid = 1;
                for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                    // Once materials have been created/added we need to then create the layers.
                    for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                        let tile_width = tileset.tile_width as f32;
                        let tile_height = tileset.tile_height as f32;

                        let _tile_space = tileset.spacing as f32; // TODO: re-add tile spacing.. :p

                        let offset_x = layer.offset_x;
                        let offset_y = layer.offset_y;

                        let mut map_settings = LayerSettings::new(
                            MapSize(
                                (tiled_map.map.width as f32 / 64.0).ceil() as u32,
                                (tiled_map.map.height as f32 / 64.0).ceil() as u32,
                            ),
                            ChunkSize(64, 64),
                            TileSize(tile_width, tile_height),
                            TextureSize(
                                tileset.image.as_ref().unwrap().width as f32,
                                tileset.image.as_ref().unwrap().height as f32,
                            ),
                        );
                        map_settings.grid_size = Vec2::new(
                            tiled_map.map.tile_width as f32,
                            tiled_map.map.tile_height as f32,
                        );

                        map_settings.mesh_type = match tiled_map.map.orientation {
                            tiled::Orientation::Hexagonal => {
                                TilemapMeshType::Hexagon(HexType::Row) // TODO: Support hex for real.
                            }
                            tiled::Orientation::Isometric => {
                                TilemapMeshType::Isometric(IsoType::Diamond)
                            }
                            tiled::Orientation::Staggered => {
                                TilemapMeshType::Isometric(IsoType::Staggered)
                            }
                            tiled::Orientation::Orthogonal => TilemapMeshType::Square,
                        };

                        // let mut debug_boxes: Vec<SpriteBundle> = vec![];
                        let mut colliders: Vec<(Collider, Transform)> = vec![];
                        let mut players: Vec<PlayerBundle> = vec![];
                        let mut cell_towers: Vec<(CellTower, Transform)> = vec![];
                        let mut camera_anchors: Vec<(CameraAnchor, Transform)> = vec![];
                        let mut boxes: Vec<BoxBundle> = vec![];

                        let layer_entity = LayerBuilder::<TileBundle>::new_batch(
                            &mut commands,
                            map_settings,
                            &mut meshes,
                            tiled_map.tilesets.get(&tileset_index).unwrap().clone_weak(),
                            0u16,
                            layer_index as u16,
                            |mut tile_pos| {
                                if tile_pos.0 >= tiled_map.map.width
                                    || tile_pos.1 >= tiled_map.map.height
                                {
                                    return None;
                                }

                                if tiled_map.map.orientation == tiled::Orientation::Orthogonal {
                                    tile_pos.1 = (tiled_map.map.height - 1) as u32 - tile_pos.1;
                                }
                                let adjustment =
                                    Vec3::new(0.0, tiled_map.map.height as f32 - 1.0, 0.0)
                                        * BLOCK_SIZE;
                                let x = tile_pos.0 as i32;
                                let y = tile_pos.1 as i32;

                                let default_transform = Transform {
                                    translation: Vec3::new(
                                        (BLOCK_SIZE * (x as f32))
                                            + (BLOCK_SIZE / 2.0),
                                        -(BLOCK_SIZE * (y as f32))
                                            + (BLOCK_SIZE / 2.0),
                                        1.0,
                                    ) + adjustment,
                                    ..default()
                                };

                                match layer.layer_type() {
                                    tiled::LayerType::TileLayer(tile_layer) => {
                                        tile_layer.get_tile(x, y).and_then(|tile| {
                                            if tile.tileset_index() != tileset_index {
                                                return None;
                                            }

                                            let gid = first_gid + tile.id();

                                            // println!("{} {} ({}, {})", tileset.name, gid, x, y);
                                            if layer.name != "Background" {
                                                match gid {
                                                    25 => cell_towers.push((
                                                        CellTower,
                                                        default_transform.clone(),
                                                    )),
                                                    27 => players.push(PlayerBundle::new(
                                                        default_transform.clone(),
                                                        player_texture_res.0.clone_weak(),
                                                    )),
                                                    31 => camera_anchors.push((
                                                        CameraAnchor,
                                                        default_transform.clone()
                                                    )),
                                                    32 => boxes.push(BoxBundle {
                                                        sprite_bundle: SpriteBundle {
                                                            texture: asset_server.load("box.png"),
                                                            transform: default_transform.clone(),
                                                            sprite: Sprite{
                                                                custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                                                                ..default()
                                                            },
                                                            ..default()
                                                        },
                                                        collider: Collider {
                                                            kind: ColliderKind::Movable,
                                                            size: Vec2::new(BLOCK_SIZE, BLOCK_SIZE),
                                                            on_ground: false,
                                                        },
                                                        gravity: Gravity::default(),
                                                        velocity: Velocity::default(),
                                                    }),
                                                    26 | 10 => (),
                                                    _ => colliders.push((
                                                        Collider {
                                                            size: Vec2::new(BLOCK_SIZE, BLOCK_SIZE),
                                                            kind: ColliderKind::Solid,
                                                            on_ground: false,
                                                        },
                                                        default_transform.clone(),
                                                    )),
                                                };
                                            }

                                            let tile = Tile {
                                                texture_index: tile.id() as u16,
                                                flip_x: tile.flip_h,
                                                flip_y: tile.flip_v,
                                                flip_d: tile.flip_d,
                                                ..default()
                                            };
                                            match gid {
                                                27 | 31 | 32 => None,
                                                _ => Some(TileBundle { tile, ..default() }),
                                            }
                                        })
                                    }
                                    _ => panic!("Unsupported layer type"),
                                }
                            },
                        );

                        commands.entity(layer_entity).insert(Transform::from_xyz(
                            offset_x,
                            -offset_y,
                            layer_index as f32,
                        ));
                        map.add_layer(&mut commands, layer_index as u16, layer_entity);
                        // commands.spawn_batch(debug_boxes);
                        commands.spawn_batch(colliders);
                        commands.spawn_batch(cell_towers);
                        commands.spawn_batch(players);
                        commands.spawn_batch(camera_anchors);
                        commands.spawn_batch(boxes);
                    }
                    first_gid += tileset.tilecount;
                }
            }
        }
    }
}

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST;
                }
            }
            _ => (),
        }
    }
}
