use std::{collections::HashMap, sync::Arc};

use eyre::Result;
use mmo_common::room::{RoomId, TileIndex};
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::room_state::{RoomMap, RoomMapLayer};

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
    let mut layers = vec![];
    let mut size = Vector2::new(0, 0);

    for ldtk_layer in &ldtk_level.layer_instances {
        if !ldtk_layer.grid_tiles.is_empty() {
            let layer_size = Vector2::new(ldtk_layer.width, ldtk_layer.height);
            size = size.zip_map(&layer_size, |a, b| a.max(b));

            let layer = convert_layer(ldtk_map, ldtk_layer)?;
            layers.push(layer);
        }
    }
    let collisions = collect_collisions(ldtk_map, &layers)?;

    Ok(RoomMap { size, layers, collisions, portals: vec![] })
}

fn convert_layer(ldtk_map: &LdtkMap, ldtk_layer: &LdtkLayerInstance) -> Result<RoomMapLayer> {
    let mut tiles = vec![TileIndex(0); (ldtk_layer.width * ldtk_layer.height) as usize];
    for tile in &ldtk_layer.grid_tiles {
        if tile.px[0] % ldtk_map.default_grid_size != 0
            || tile.px[1] % ldtk_map.default_grid_size != 0
        {
            return Err(eyre::eyre!(
                "Tile position is not a multiple of the default grid size"
            ));
        }

        let x = tile.px[0] / ldtk_map.default_grid_size;
        let y = tile.px[1] / ldtk_map.default_grid_size;
        tiles[(y * ldtk_layer.width + x) as usize] = tile.t;
    }
    Ok(RoomMapLayer { tiles })
}

fn collect_collisions(ldtk_map: &LdtkMap, layers: &[RoomMapLayer]) -> Result<Vec<bool>> {
    let tileset = ldtk_map.defs.tilesets.first().ok_or_else(|| eyre::eyre!("No first tileset"))?;
    let blocked_tile_ids = tileset.enum_tags.iter().find(|tag| tag.enum_value_id == "Blocked");

    let mut collisions =
        vec![false; (ldtk_map.default_grid_size * ldtk_map.default_grid_size) as usize];

    if let Some(blocked_tile_ids) = blocked_tile_ids {
        for layer in layers {
            for (i, tile) in layer.tiles.iter().enumerate() {
                collisions[i] = blocked_tile_ids.tile_ids.contains(tile);
            }
        }
    }
    Ok(collisions)
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
