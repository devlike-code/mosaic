use std::sync::Arc;

use grouping::GroupingCapability;

use crate::{
    capabilities::{grouping, ArchetypeSubject, StringCapability},
    internals::{pars, ComponentValuesBuilderSetter, MosaicIO, MosaicTypelevelCRUD, Tile},
};

pub fn generate_enum(enum_tile: &Tile) -> Tile {
    let mosaic = Arc::clone(&enum_tile.mosaic);
    mosaic
        .new_type("Error: { message: s32, target: u64 };")
        .unwrap();

    match generate_enum_code(enum_tile) {
        Ok(code) => mosaic.create_string_object(code.as_str()).unwrap(),
        Err((str, target)) => mosaic.new_object(
            "Error",
            pars()
                .set("message", str.as_str())
                .set("target", target.id as u64)
                .ok(),
        ),
    }
}

pub fn generate_enum_code(enum_tile: &Tile) -> Result<String, (String, Tile)> {
    let mut builder = "".to_string();
    let mosaic = Arc::clone(&enum_tile.mosaic);
    if let Some(name) = enum_tile.get_component("Label") {
        builder += format!("internal enum E{} {{\n", name.get("self").as_s32()).as_str();
        if mosaic.get_group_owner("Enum", enum_tile).is_none() {
            return Err((
                format!("Missing enum group component on #{}.", enum_tile.id),
                enum_tile.clone(),
            ));
        }
        for member in mosaic.get_group_members("Enum", enum_tile) {
            if let Some(member_name) = member.get_component("Label") {
                builder += format!("\t{},\n", member_name.get("self").as_s32()).as_str();
            } else {
                return Err((
                    format!("Missing label on tile #{}.", member.id),
                    member.clone(),
                ));
            }
        }
        builder += "}\n";
    } else {
        return Err((
            format!("Missing label on tile #{}.", enum_tile),
            enum_tile.clone(),
        ));
    }

    Ok(builder)
}

#[cfg(test)]
mod primitive_code_gen_tests {
    use crate::{
        capabilities::GroupingCapability,
        internals::{par, Mosaic, MosaicIO, MosaicTypelevelCRUD},
    };

    use super::generate_enum;

    #[test]
    fn test_enums() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Label: s32;").unwrap();
        let a = mosaic.new_object("Label", par("Variant"));
        let b = mosaic.new_object("Label", par("Other"));
        let c = mosaic.new_object("Label", par("Third"));
        let e = mosaic.new_object("Label", par("MyEnum"));
        mosaic.group("Enum", &e, &[a, b, c]);

        println!("\n{}", generate_enum(&e));
    }
}
