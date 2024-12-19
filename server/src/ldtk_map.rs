use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use eyre::Result;
use mmo_common::room::{ForegroundTile, RoomId, TileIndex};
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::room_state::RoomMap;

pub fn load(path: &str) -> Result<HashMap<RoomId, Arc<RoomMap>>> {
    let json = std::fs::read_to_string(path)?;
    let ldtk_map: LdtkMap = serde_json::from_str(&json)?;

    let maps = ldtk_map
        .levels
        .iter()
        .enumerate()
        .map(|(i, ldtk_level)| {
            let map = convert_map(&ldtk_map, ldtk_level)?;
            let map = Arc::new(map);
            Ok((RoomId(i as u64), map))
        })
        .collect();

    maps
}

fn convert_map(ldtk_map: &LdtkMap, ldtk_level: &LdtkLevel) -> Result<RoomMap> {
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
    }
    let collisions = collect_collisions(
        size,
        &bg_dense_layers,
        &bg_sparse_layer,
        &fg_sparse_layer,
        &blocked_tile_ids,
    );

    Ok(RoomMap {
        size,
        bg_dense_layers,
        bg_sparse_layer,
        fg_sparse_layer,
        collisions,
        portals: vec![],
    })
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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtkTileInstance {
    px: [u32; 2],
    t: TileIndex,
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
