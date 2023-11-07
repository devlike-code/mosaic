use std::collections::HashSet;

use crate::{
    ec::{has_component, EntityId},
    graph::{get_graph_refs, get_graph_storage},
    protocol::{component_name, v1::get_type},
    sparse_matrix::Matrix,
    storage::get_type_by_name_storage,
    top_level::{
        get_brick, get_brick_count, get_comp_field_i32, get_comp_field_name, get_frame_for_entity,
        get_framed_entities, get_graph_for_entity, get_named_type, Block, Brick,
    },
};

pub fn validate_type_exists(name: &str) -> Result<(), String> {
    if !get_type_by_name_storage().contains_key(&component_name(name)) {
        Err(format!("Type '{}' not registered.", name.to_string()))
    } else {
        Ok(())
    }
}

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

pub fn validate_frame_is_populated(frame_id: EntityId) -> Result<(), String> {
    if let Some(frame) = get_graph_storage().get(&frame_id) {
        if !frame.is_empty() {
            Ok(())
        } else {
            Err(format!("Frame {} is empty.", frame_id))
        }
    } else {
        Err(format!(
            "Frame {} is not present in the graph storage.",
            frame_id
        ))
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
