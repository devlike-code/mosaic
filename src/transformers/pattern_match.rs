use std::{
    collections::HashMap,
    sync::Arc,
};

use array_tool::vec::Intersect;
use itertools::Itertools;
use ordered_multimap::ListOrderedMultimap;

use crate::{
    capabilities::{
        process::ProcessCapability, SelectionCapability,
        TraversalOperator, Traverse,
    },
    internals::{
        self_val,
        EntityId, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD, Tile, TileFieldGetter, Value,
    },
};

fn chr(i: usize) -> char {
    char::from_u32(65 + i as u32).unwrap()
}

// fn chr2(i: (usize, usize)) -> (char, char) {
//     (chr(i.0), chr(i.1))
// }

// fn get_candidate_pairs(
//     candidates: &ListOrderedMultimap<EntityId, EntityId>,
//     pattern_node: &Tile,
// ) -> Vec<EntityId> {
//     candidates.get_all(&pattern_node.id).copied().collect_vec()
// }

#[derive(Default)]
pub(crate) struct PatternMatchState {
    candidates: ListOrderedMultimap<EntityId, EntityId>,
    in_degree_mmap: ListOrderedMultimap<EntityId, EntityId>,
    out_degree_mmap: ListOrderedMultimap<EntityId, EntityId>,
}

fn find_candidates_by_degrees(
    pattern: &TraversalOperator,
    target: &TraversalOperator,
) -> PatternMatchState {
    let mut state = PatternMatchState::default();

    for target_node in target.get_objects() {
        let in_degree = target.in_degree(&target_node);
        let out_degree = target.out_degree(&target_node);

        for i in 0..=in_degree {
            state.in_degree_mmap.append(i, target_node.id);
        }

        for i in 0..=out_degree {
            state.out_degree_mmap.append(i, target_node.id);
        }
    }

    for pattern_node in pattern.get_objects() {
        let in_degree = pattern.in_degree(&pattern_node);
        let out_degree = pattern.out_degree(&pattern_node);

        let in_candidates = state.in_degree_mmap.get_all(&in_degree).collect_vec();
        let out_candidates = state.out_degree_mmap.get_all(&out_degree).collect_vec();

        in_candidates
            .intersect(out_candidates)
            .into_iter()
            .for_each(|target_node| {
                state.candidates.append(pattern_node.id, *target_node);
            });
    }

    state
}

pub fn pattern_match(match_process: &Tile) -> anyhow::Result<Tile> {
    let mosaic = Arc::clone(&match_process.mosaic);
    mosaic.new_type("PatternMatch_Candidate: s32; PatternMatch_Binding: s32;")?;

    let pattern_param = mosaic
        .get_process_parameter_value(match_process, "pattern")?
        .unwrap();

    let target_param = mosaic
        .get_process_parameter_value(match_process, "target")?
        .unwrap();

    let pattern_tiles_iter = mosaic.get_selection(&pattern_param);
    let target_tiles_iter = mosaic.get_selection(&target_param);

    let pattern = mosaic.traverse(pattern_tiles_iter.into());
    let target = mosaic.traverse(target_tiles_iter.into());

    let state = find_candidates_by_degrees(&pattern, &target);

    let reachability = target.as_matrix();

    let mut candidate_mapping = HashMap::new();
    let mut rev_candidate_mapping = HashMap::new();

    let mut transient = vec![];

    for start_node in pattern.get_objects() {
        let pid = start_node.id;
        let start_candidates = state.candidates.get_all(&start_node.id).collect_vec();

        for &sc in &start_candidates {
            let candidate = mosaic.new_object(
                "PatternMatch_Candidate",
                self_val(Value::S32(format!("{:}:{:}", chr(pid), chr(*sc)).into())),
            );

            candidate_mapping.insert(candidate.id, (pid, *sc));
            rev_candidate_mapping.insert((pid, *sc), candidate.id);
            transient.push(candidate);
        }
    }

    for start_node in pattern.get_objects() {
        let pid = start_node.id;
        let start_candidates = state.candidates.get_all(&start_node.id).collect_vec();

        for end_node in pattern.get_forward_neighbors(&start_node) {
            let tid = end_node.id;
            let end_candidates = state.candidates.get_all(&end_node.id).collect_vec();

            for &sc in &start_candidates {
                for &ec in &end_candidates {
                    if *sc == *ec {
                        continue;
                    }

                    if !reachability.are_adjacent(*sc, *ec) {
                        continue;
                    }

                    let cand1 = rev_candidate_mapping.get(&(pid, *sc)).unwrap();
                    let cand2 = rev_candidate_mapping.get(&(tid, *ec)).unwrap();

                    let binding = mosaic.new_arrow(
                        cand1,
                        cand2,
                        "PatternMatch_Binding",
                        self_val(Value::S32(
                            format!("{:}:{:} -> {:}:{:}", chr(pid), chr(*sc), chr(tid), chr(*ec))
                                .into(),
                        )),
                    );

                    transient.push(binding);
                }
            }
        }
    }

    transient.iter().for_each(|e| println!("{:?}", e));

    let transient_target = mosaic.traverse(transient.into());
    let state2 = find_candidates_by_degrees(&pattern, &transient_target);

    println!(
        "{:?}",
        state2.candidates.into_iter().for_each(|(a, b)| {
            println!("{:?} = {:?}", chr(a), mosaic.get(b).unwrap().get("self"));
        })
    );
    Ok(match_process.clone())
}

#[cfg(test)]
mod pattern_match_tests {
    use crate::{
        capabilities::{process::ProcessCapability, SelectionCapability},
        internals::{default_vals, Mosaic, MosaicCRUD, MosaicIO},
    };

    use super::pattern_match;

    #[test]
    fn test_pattern_match() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("DEBUG", default_vals()); // 0
        let b = mosaic.new_object("DEBUG", default_vals()); // 1
        let c = mosaic.new_object("DEBUG", default_vals()); // 2
        mosaic.new_arrow(&a, &b, "DEBUG", default_vals()); // 3
        mosaic.new_arrow(&a, &c, "DEBUG", default_vals()); // 4
        mosaic.new_arrow(&b, &c, "DEBUG", default_vals()); // 5

        let g = mosaic.new_object("DEBUG", default_vals()); // 6
        let h = mosaic.new_object("DEBUG", default_vals()); // 7
        let i = mosaic.new_object("DEBUG", default_vals()); // 8
        let j = mosaic.new_object("DEBUG", default_vals()); // 9
        let k = mosaic.new_object("DEBUG", default_vals()); // 10
        mosaic.new_arrow(&g, &h, "DEBUG", default_vals()); // 11
        mosaic.new_arrow(&g, &i, "DEBUG", default_vals()); // 12
        mosaic.new_arrow(&h, &i, "DEBUG", default_vals()); // 13
        mosaic.new_arrow(&g, &j, "DEBUG", default_vals()); // 14
        mosaic.new_arrow(&i, &j, "DEBUG", default_vals()); // 15
        mosaic.new_arrow(&h, &k, "DEBUG", default_vals()); // 16

        let p = mosaic.make_selection();
        mosaic.fill_selection(&p, &[&a, &b, &c]);

        let t = mosaic.make_selection();
        mosaic.fill_selection(&t, &[&g, &h, &i, &j, &k]);

        let mtch = mosaic
            .create_process("PatternMatch", &["pattern", "target"])
            .unwrap();
        mosaic.pass_process_parameter(&mtch, "pattern", &p).unwrap();
        mosaic.pass_process_parameter(&mtch, "target", &t).unwrap();
        pattern_match(&mtch).unwrap();
    }
}
