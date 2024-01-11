use std::sync::Arc;

use crate::{
    capabilities::{process::ProcessCapability, ArchetypeSubject, StringCapability},
    internals::{pars, ComponentValuesBuilderSetter, Mosaic, MosaicIO, MosaicTypelevelCRUD, Tile},
    iterators::tile_getters::TileGetters,
};

// #[derive(Transformer)]
pub fn generate_enum(process_tile: &Tile) -> Tile {
    let mosaic = Arc::clone(&process_tile.mosaic);
    mosaic
        .new_type("Error: { message: s128, target: u64 };")
        .unwrap();

    if let Some((_, Some(enum_tile))) = mosaic
        .get_process_parameter_values(process_tile)
        .unwrap()
        .iter()
        .next()
    {
        match generate_enum_code(enum_tile) {
            Ok(code) => mosaic.create_string_object(code.as_str()).unwrap(),
            Err((str, target)) => mosaic.new_object(
                "Error",
                pars()
                    .set("message", str.as_bytes())
                    .set("target", target.id as u64)
                    .ok(),
            ),
        }
    } else {
        mosaic.new_object(
            "Error",
            pars()
                .set(
                    "message",
                    "Cannot find enum tile - none passed as argument.",
                )
                .set("target", process_tile.id as u64)
                .ok(),
        )
    }
}

pub fn option_use_csharp_enum_naming_convention(enum_tile: &Tile) -> String {
    if enum_tile
        .get_component("CodeUseCSharpNamingConvention")
        .is_some()
    {
        "E"
    } else {
        ""
    }
    .to_string()
}

pub fn option_indent_with_spaces(enum_tile: &Tile) -> String {
    if enum_tile.get_component("CodeIndentWithSpaces").is_some() {
        "  "
    } else {
        "\t"
    }
    .to_string()
}

fn validate_enum(mosaic: &Arc<Mosaic>, enum_tile: &Tile) -> Result<(), (String, Tile)> {
    if enum_tile.get_component("Enum").is_none() {
        return Err((
            format!("Missing Enum component on #{}.", enum_tile.id),
            enum_tile.clone(),
        ));
    }

    if enum_tile.get_component("Label").is_none() {
        return Err((
            format!("Missing label on tile #{}.", enum_tile.id),
            enum_tile.clone(),
        ));
    }

    for member in enum_tile.iter().get_arrows_from().get_targets() {
        if member.get_component("Label").is_none() {
            return Err((
                format!("Missing label on tile #{}.", member.id),
                member.clone(),
            ));
        }
    }

    Ok(())
}

pub fn generate_enum_code(enum_tile: &Tile) -> Result<String, (String, Tile)> {
    let mut builder = "".to_string();
    let mosaic = Arc::clone(&enum_tile.mosaic);

    let spacing = option_indent_with_spaces(enum_tile);
    let enum_naming = option_use_csharp_enum_naming_convention(enum_tile);

    validate_enum(&mosaic, enum_tile)?;

    if let Some(name) = enum_tile.get_component("Label") {
        builder += format!(
            "internal enum {}{} {{\n",
            enum_naming,
            name.get("self").as_s32()
        )
        .as_str();

        for member in enum_tile.iter().get_arrows_from().get_targets() {
            let member_name = member.get_component("Label").unwrap();
            builder += format!("{}{},\n", spacing, member_name.get("self").as_s32()).as_str();
        }
        builder += "}\n";
    }

    Ok(builder)
}

#[cfg(test)]
mod primitive_code_gen_tests {
    use crate::{
        capabilities::{process::ProcessCapability, StringCapability},
        internals::{par, void, Mosaic, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD},
    };

    use super::generate_enum;

    #[test]
    fn test_enums() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Arrow: unit;").unwrap();
        mosaic.new_type("Label: s32;").unwrap();
        mosaic.new_type("Enum: s32;").unwrap();
        mosaic.new_type("CodeIndentWithSpaces: unit;").unwrap();
        mosaic
            .new_type("CodeUseCSharpNamingConvention: unit;")
            .unwrap();
        let a = mosaic.new_object("Label", par("Variant"));
        let b = mosaic.new_object("Label", par("Other"));
        let c = mosaic.new_object("Label", par("Third"));
        let e = mosaic.new_object("Label", par("MyEnum"));
        mosaic.new_descriptor(&e, "Enum", par("MyEnum"));
        for i in &[a, b, c] {
            mosaic.new_arrow(&e, i, "Arrow", void());
        }

        mosaic.new_descriptor(&e, "CodeIndentWithSpaces", void());
        mosaic.new_descriptor(&e, "CodeUseCSharpNamingConvention", void());

        let p = mosaic.create_process("generate_enum", &["input"]).unwrap();
        mosaic.pass_process_parameter(&p, "input", &e).unwrap();
        println!("{}", mosaic.dot(""));

        let r = generate_enum(&p);
        assert!(r.component.is("String"));
        println!("{}", mosaic.get_string_value(&r).unwrap());
    }
}
