use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use eyre::Result;
use mmo_common::room::{ForegroundTile, RoomId, TileIndex};
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::{
    room_state::{MobSpawn, Portal, RoomMap},
    server_context::World,
};

pub fn load(path: &str) -> Result<World> {
    let json = std::fs::read_to_string(path)?;
    let ldtk_map: LdtkMap = serde_json::from_str(&json)?;

    let maps: Result<Vec<ParsedMap>> = ldtk_map
        .levels
        .iter()
        .map(|ldtk_level| convert_map(&ldtk_map, ldtk_level))
        .collect();

    let mut maps = maps?;
    resolve_portals(&mut maps)?;

    let (start_room_id, start_position) = find_start_position(&maps)?;
    let start_position = start_position.cast().add_scalar(0.5);

    let maps = maps
        .into_iter()
        .enumerate()
        .map(|(i, map)| (RoomId(i as u64), Arc::new(map.map)))
        .collect();

    Ok(World { maps, start_room_id, start_position })
}

struct ParsedMap {
    map: RoomMap,
    portals: Vec<ParsedPortal>,
    player_starts: Vec<Vector2<u32>>,
}

fn convert_map(ldtk_map: &LdtkMap, ldtk_level: &LdtkLevel) -> Result<ParsedMap> {
    let foreground_tile_ids: HashMap<TileIndex, u32> = {
        let h1 = collect_enum_tile_ids(ldtk_map, "Foreground_h1")?;
        let h2 = collect_enum_tile_ids(ldtk_map, "Foreground_h2")?;
        [(h1, 1), (h2, 2)]
            .into_iter()
            .flat_map(|(set, h)| set.into_iter().map(move |id| (id, h)))
            .collect()
    };
    let blocked_tile_ids = collect_enum_tile_ids(ldtk_map, "Blocked")?;

    let mut size = Vector2::new(0, 0);
    let mut bg_dense_layers = vec![];
    let mut bg_sparse_layer = vec![];
    let mut fg_sparse_layer = vec![];
    let mut mob_spawns = vec![];
    let mut portals = vec![];
    let mut player_starts = vec![];

    for ldtk_layer in &ldtk_level.layer_instances {
        if !ldtk_layer.grid_tiles.is_empty() {
            let layer_size = Vector2::new(ldtk_layer.width, ldtk_layer.height);
            size = size.zip_map(&layer_size, |a, b| a.max(b));

            let has_non_divisible_pos = ldtk_layer.grid_tiles.iter().any(|tile| {
                tile.px[0] % ldtk_map.default_grid_size != 0
                    || tile.px[1] % ldtk_map.default_grid_size != 0
            });
            if has_non_divisible_pos {
                return Err(eyre::eyre!(
                    "Tile position is not a multiple of the default grid size"
                ));
            }

            let has_duplicate_positions = {
                let unique_positions = ldtk_layer
                    .grid_tiles
                    .iter()
                    .map(|tile| tile.px)
                    .collect::<std::collections::HashSet<_>>();
                unique_positions.len() != ldtk_layer.grid_tiles.len()
            };
            let has_foreground_tiles = ldtk_layer
                .grid_tiles
                .iter()
                .any(|tile| foreground_tile_ids.contains_key(&tile.t));

            if has_duplicate_positions || has_foreground_tiles {
                let (bg, fg) = convert_sparse_layer(ldtk_map, ldtk_layer, &foreground_tile_ids);
                bg_sparse_layer.extend(bg);
                fg_sparse_layer.extend(fg);
            } else {
                let layer = convert_dense_layer(ldtk_map, ldtk_layer);
                bg_dense_layers.push(layer);
            }
        }

        if !ldtk_layer.entity_instances.is_empty() {
            for entity in collect_entities(&ldtk_layer.entity_instances) {
                match entity {
                    ParsedEntity::MobSpawn(mob_spawn) => mob_spawns.push(Arc::new(mob_spawn)),
                    ParsedEntity::Portal(portal) => portals.push(portal),
                    ParsedEntity::PlayerStart(position) => player_starts.push(position),
                }
            }
        }
    }
    let collisions = collect_collisions(
        size,
        &bg_dense_layers,
        &bg_sparse_layer,
        &fg_sparse_layer,
        &blocked_tile_ids,
    );

    Ok(ParsedMap {
        map: RoomMap {
            size,
            bg_dense_layers,
            bg_sparse_layer,
            fg_sparse_layer,
            collisions,
            portals: vec![],
            mob_spawns,
        },
        portals,
        player_starts,
    })
}

fn resolve_portals(maps: &mut Vec<ParsedMap>) -> Result<()> {
    let portals: HashMap<String, (usize, Vector2<u32>)> = maps
        .iter()
        .enumerate()
        .flat_map(|(i, map)| {
            map.portals
                .iter()
                .map(move |portal| (portal.entity_iid.clone(), (i, portal.position)))
        })
        .collect();

    for map in maps {
        let map_portals: Result<Vec<Portal>> = map
            .portals
            .iter()
            .map(|portal| {
                if let Some((target_map, target_position)) = portals.get(&portal.target_entity_iid)
                {
                    Ok(Portal {
                        position: portal.position,
                        target_room_id: RoomId(*target_map as u64),
                        target_position: target_position.cast(),
                    })
                } else {
                    Err(eyre::eyre!(format!(
                        "Portal target not found: {}",
                        portal.entity_iid
                    )))
                }
            })
            .collect();
        map.map.portals = map_portals?;
    }
    Ok(())
}

fn convert_sparse_layer(
    ldtk_map: &LdtkMap,
    ldtk_layer: &LdtkLayerInstance,
    foreground_tile_ids: &HashMap<TileIndex, u32>,
) -> (Vec<(Vector2<u32>, TileIndex)>, Vec<ForegroundTile>) {
    let grid_size = ldtk_map.default_grid_size;
    let mut bg = vec![];
    let mut fg = vec![];
    for tile in &ldtk_layer.grid_tiles {
        let position = Vector2::new(tile.px[0] / grid_size, tile.px[1] / grid_size);
        if let Some(height) = foreground_tile_ids.get(&tile.t) {
            fg.push(ForegroundTile { position, height: *height, tile_index: tile.t });
        } else {
            bg.push((position, tile.t));
        }
    }
    (bg, fg)
}

fn convert_dense_layer(ldtk_map: &LdtkMap, ldtk_layer: &LdtkLayerInstance) -> Vec<TileIndex> {
    let grid_size = ldtk_map.default_grid_size;
    let mut tiles = vec![TileIndex::empty(); (ldtk_layer.width * ldtk_layer.height) as usize];
    for tile in &ldtk_layer.grid_tiles {
        let x = tile.px[0] / grid_size;
        let y = tile.px[1] / grid_size;
        tiles[(y * ldtk_layer.width + x) as usize] = tile.t;
    }
    tiles
}

fn collect_collisions(
    size: Vector2<u32>,
    dense_layers: &[Vec<TileIndex>],
    bg_sparse_layer: &Vec<(Vector2<u32>, TileIndex)>,
    fg_sparse_layer: &Vec<ForegroundTile>,
    blocked_tile_ids: &HashSet<TileIndex>,
) -> Vec<bool> {
    let mut collisions = vec![false; (size.x * size.y) as usize];
    for layer in dense_layers {
        for (i, tile) in layer.iter().enumerate() {
            if blocked_tile_ids.contains(tile) {
                collisions[i] = true;
            }
        }
    }
    for (position, tile) in bg_sparse_layer {
        if blocked_tile_ids.contains(tile) {
            let i = position.y * size.x + position.x;
            collisions[i as usize] = true;
        }
    }
    for fg_tile in fg_sparse_layer {
        if blocked_tile_ids.contains(&fg_tile.tile_index) {
            let i = fg_tile.position.y * size.x + fg_tile.position.x;
            collisions[i as usize] = true;
        }
    }
    collisions
}

fn collect_enum_tile_ids(ldtk_map: &LdtkMap, enum_value: &str) -> Result<HashSet<TileIndex>> {
    let tileset = ldtk_map.defs.tilesets.first().ok_or_else(|| eyre::eyre!("No first tileset"))?;
    if let Some(enum_tags) = tileset.enum_tags.iter().find(|tag| tag.enum_value_id == enum_value) {
        Ok(enum_tags.tile_ids.iter().copied().collect())
    } else {
        Ok(HashSet::new())
    }
}

fn collect_entities(entities: &[LdtkEntityInstance]) -> Vec<ParsedEntity> {
    entities.iter().filter_map(collect_entity).collect()
}

fn collect_entity(entity: &LdtkEntityInstance) -> Option<ParsedEntity> {
    match entity.identifier.as_str() {
        "Mob" => match entity.field("mob")? {
            LdtkEntityFieldInstance::String { value, .. } => {
                Some(ParsedEntity::MobSpawn(MobSpawn {
                    position: entity.grid,
                    mob_template: value.clone(),
                }))
            }
            _ => None,
        },
        "Portal" => match entity.field("target")? {
            LdtkEntityFieldInstance::EntityRef { value, .. } => {
                Some(ParsedEntity::Portal(ParsedPortal {
                    position: entity.grid,
                    entity_iid: entity.iid.clone(),
                    target_entity_iid: value.entity_iid.clone(),
                }))
            }
            _ => None,
        },
        "Player_Start" => Some(ParsedEntity::PlayerStart(entity.grid)),
        _ => None,
    }
}

fn find_start_position(maps: &[ParsedMap]) -> Result<(RoomId, Vector2<u32>)> {
    let player_start_entities = maps
        .iter()
        .enumerate()
        .flat_map(|(i, map)| {
            map.player_starts.iter().map(move |position| (RoomId(i as u64), *position))
        })
        .collect::<Vec<_>>();
    match player_start_entities.as_slice() {
        &[(room_id, position)] => Ok((room_id, position)),
        &[] => Err(eyre::eyre!("No player start found")),
        _ => Err(eyre::eyre!("Multiple player starts found")),
    }
}

#[derive(Debug, Clone)]
enum ParsedEntity {
    MobSpawn(MobSpawn),
    Portal(ParsedPortal),
    PlayerStart(Vector2<u32>),
}

#[derive(Debug, Clone)]
struct ParsedPortal {
    position: Vector2<u32>,
    entity_iid: String,
    target_entity_iid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtkMap {
    default_grid_size: u32,
    defs: LdtkDefs,
    levels: Vec<LdtkLevel>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtkDefs {
    tilesets: Vec<LdtkTileset>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtkLevel {
    layer_instances: Vec<LdtkLayerInstance>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtkLayerInstance {
    #[serde(rename = "__type")]
    typ: String,
    #[serde(rename = "__cWid")]
    width: u32,
    #[serde(rename = "__cHei")]
    height: u32,
    grid_tiles: Vec<LdtkTileInstance>,
    entity_instances: Vec<LdtkEntityInstance>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtkTileInstance {
    px: [u32; 2],
    t: TileIndex,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtkEntityInstance {
    #[serde(rename = "__identifier")]
    identifier: String,
    #[serde(rename = "__grid")]
    grid: Vector2<u32>,
    iid: String,
    field_instances: Vec<LdtkEntityFieldInstance>,
}

impl LdtkEntityInstance {
    pub fn field(&self, identifier: &str) -> Option<&LdtkEntityFieldInstance> {
        self.field_instances.iter().find(|field| match field {
            LdtkEntityFieldInstance::String { identifier: id, .. } => id == identifier,
            LdtkEntityFieldInstance::EntityRef { identifier: id, .. } => id == identifier,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "__type")]
enum LdtkEntityFieldInstance {
    String {
        #[serde(rename = "__identifier")]
        identifier: String,
        #[serde(rename = "__value")]
        value: String,
    },
    EntityRef {
        #[serde(rename = "__identifier")]
        identifier: String,
        #[serde(rename = "__value")]
        value: LdtkEntityRef,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtkEntityRef {
    entity_iid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtkTileset {
    enum_tags: Vec<LdtkEnumTag>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtkEnumTag {
    enum_value_id: String,
    tile_ids: Vec<TileIndex>,
}
