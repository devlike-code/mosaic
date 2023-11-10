use std::sync::Arc;

use crate::{
    internals::{mosaic_engine::MosaicEngine, Tile},
    layers::{indirection::Indirection, parenting::Parenting},
};

/// .
///
/// # Errors
///
/// This function will return an error if .
#[allow(dead_code)]
pub(crate) fn validate_tile_is_arrow(t: &Tile) -> Result<&Tile, String> {
    if !t.is_arrow() {
        Err("Block is required to have exactly arrow.".to_string())
    } else {
        Ok(t)
    }
}

/// .
///
/// # Errors
///
/// This function will return an error if .
#[allow(dead_code)]
pub(crate) fn validate_arrow_is_graph_match(
    t: &Tile,
    engine_state: Arc<MosaicEngine>,
) -> Result<(), String> {
    let len = engine_state
        .build_query()
        .with_source(t.id())
        .with_component("GraphMatch".into())
        .get()
        .len();

    if len != 1 {
        Err("[graph_match.rs][validate_arrow_is_graph_match] Arrow requires to have the GraphMatch component.".to_string())
    } else {
        Ok(())
    }
}

pub fn validate_type_exists(name: &str, engine: Arc<MosaicEngine>) -> Result<(), String> {
    if !engine.engine_state.has_component_type(&name.into()) {
        Err(format!("Type '{}' not registered.", name))
    } else {
        Ok(())
    }
}

pub fn validate_frame_is_populated(
    parent: Tile,
    engine_state: Arc<MosaicEngine>,
) -> Result<(), String> {
    let children = engine_state.get_children(&parent);
    if !children.is_empty() {
        Ok(())
    } else {
        Err(format!("Frame {} is empty.", parent.id()))
    }
}
/*
pub fn validate_first_brick_in_block(b: &Block) -> Result<Brick, String> {
    if get_brick_count(&b) >= 1 {
        Ok(get_brick(&b, 0).clone())
    } else {
        Err(format!("Block required to contain at least a single brick"))
    }
}

pub fn validate_entity_has_position(eid: EntityId) -> Result<(i32, i32), String> {
    if has_component(eid, "Position") {
        let x = get_comp_field_i32(eid, "Position", "Position.x")?;
        let y = get_comp_field_i32(eid, "Position", "Position.y")?;
        Ok((x, y))
    } else {
        Err(format!("No position component found on entity {}", eid))
    }
}

pub fn validate_frame_has_label(frame_id: EntityId) -> Result<(), String> {
    if !has_component(frame_id, "Label") {
        Err(format!("Label component not found in frame {}", frame_id))
    } else {
        Ok(())
    }
}

pub fn validate_frame_has_output_dir(frame_id: EntityId) -> Result<(), String> {
    if !has_component(frame_id, "OutputDir") {
        Err(format!(
            "OutputDir component not found in frame {}",
            frame_id
        ))
    } else {
        Ok(())
    }
}



pub fn validate_single_entry_point(frame_id: EntityId) -> Result<EntityId, String> {
    let parent = get_frame_for_entity(frame_id).unwrap();
    let frame_matrix = get_graph_for_entity(parent);
    let neighbors = frame_matrix.get_front_neighbors(frame_id);
    println!("{:?}", neighbors);
    let n = neighbors.len();
    if n > 1 {
        Err(format!(
            "Expected single entry point from frame into graph, {} found.",
            neighbors.len()
        ))
    } else if n < 1 {
        Err(format!(
            "Expected single entry point from frame into graph, none found."
        ))
    } else {
        Ok(*neighbors.first().unwrap())
    }
}

pub fn validate_all_content_labelled(frame_id: EntityId) -> Result<(), String> {
    let graph_refs = get_graph_refs().read().unwrap();
    if let Some(entities) = graph_refs.get_vec(&frame_id) {
        for entity in entities {
            // println!("EntityId = {:?} in Frame {:?}", entity, frame_id);
            if !has_component(*entity, "Label") {
                return Err(format!(
                    "Required label component missing from entity {}.",
                    *entity
                ));
            }
        }
        Ok(())
    } else {
        Err(format!("Frame {} is empty.", frame_id))
    }
}

pub fn validate_distinctly_labelled_nodes(frame_id: EntityId) -> Result<(), String> {
    let entities = get_framed_entities(frame_id);
    let label_type = get_named_type("Label")?;
    let mut name_set = HashSet::new();
    let mut node_count = 0;

    for entity in &entities {
        if !has_component(*entity, "Node") {
            continue;
        }
        node_count += 1;
        let name = get_comp_field_name(*entity, "Label", "")?;
        name_set.insert(name);
    }

    if name_set.len() != node_count {
        Err(format!("All nodes are required to have different labels."))
    } else {
        Ok(())
    }
}

pub fn validate_output_dir_exists(dir: &String) -> Result<bool, String> {
    if std::path::Path::new(&dir.trim().replace("\0", "")).is_dir() {
        Ok(true)
    } else {
        Err(format!("Path '{}' is not a valid directory.", dir))
    }
}
 */
