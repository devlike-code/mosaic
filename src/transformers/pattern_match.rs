use std::sync::Arc;

use crate::{
    capabilities::process::ProcessCapability,
    internals::{default_vals, Mosaic, Tile},
};

pub fn pattern_match(mosaic: Arc<Mosaic>, pattern: &Tile, target: &Tile) -> anyhow::Result<Tile> {
    let match_tile = mosaic.create_process("PatternMatch", &["pattern", "target"])?;
    mosaic.pass_process_parameter(&match_tile, "pattern", pattern)?;
    mosaic.pass_process_parameter(&match_tile, "target", target)?;

    // this needs to change:
    let result = mosaic.new_object("DEBUG", default_vals());

    mosaic.add_process_result(&match_tile, &result)?;
    Ok(match_tile)
}
