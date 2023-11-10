use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::internals::mosaic_engine::MosaicEngine;
use crate::internals::{Block, EntityId, Tile};

use crate::layers::indirection::Indirection;
use crate::layers::parenting::Parenting;
use crate::layers::traversing::Traversing;
use crate::transformers::validation::{
    self, validate_arrow_is_graph_match, validate_frame_is_populated, validate_tile_is_arrow,
};

use super::validation::validate_type_exists;

#[derive(Debug)]
struct GraphParameters<'a> {
    pattern_size: usize,
    candidates: &'a HashMap<EntityId, HashSet<EntityId>>,
}

fn hash_hashmap<K: Ord + Clone + Display, V: Clone + Display>(hashmap: &HashMap<K, V>) -> u64 {
    let mut sorted_tuples: Vec<(K, V)> = hashmap
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    sorted_tuples.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
    let str = (sorted_tuples
        .into_iter()
        .map(|(k, v)| format!("{},{}.", k, v))
        .collect::<Vec<String>>())
    .join("");

    let mut hasher = DefaultHasher::default();
    str.hash(&mut hasher);
    hasher.finish()
}

pub fn graph_match(input: &Tile, engine_state: Arc<MosaicEngine>) -> Result<Vec<Tile>, String> {
    let mut arrow_counters = u32::MAX;

    fn tuple_is_completely_different(a: &(u32, u32), b: &(u32, u32)) -> bool {
        a.0 != b.0 && a.1 != b.1
    }

    validate_type_exists("GraphMatch", engine_state)?;

    let arrow = validate_tile_is_arrow(input)?;
    validate_arrow_is_graph_match(arrow, engine_state)?;

    let pattern = arrow.get_endpoints().0; // staring arrow's source
    let target = arrow.get_endpoints().1; // staring arrow's target

    validate_frame_is_populated(pattern, engine_state)?;
    validate_frame_is_populated(target, engine_state)?;

    let pattern_children = engine_state.get_children(&pattern).elements; // graph where pattern is parent
    let target_children = engine_state.get_children(&target).elements; // graph where traget is parent

    let mut candidates: HashMap<EntityId, HashSet<EntityId>> = HashMap::default();

    //let archetypes = get_archetypes_by_entity().read().unwrap().clone();

    let pattern_size = pattern_children.len();

    for node in pattern_children {
        let out_count = engine_state.out_degree(node);
        let in_count = engine_state.in_degree(node);
        let node_arch = engine_state
            .get_vec(&node)
            .unwrap()
            .into_iter()
            .collect::<HashSet<_>>();

        if !candidates.contains_key(&node) {
            candidates.insert(node, HashSet::default());
        }

        for cand in target_children {
            let cand_out_count = engine_state.out_degree(cand);
            let cand_in_count = engine_state.in_degree(cand);
            if cand_out_count >= out_count && cand_in_count >= in_count {
                let cand_arch = archetypes
                    .get_vec(&cand)
                    .unwrap()
                    .into_iter()
                    .collect::<HashSet<_>>();
                if node_arch.difference(&cand_arch).count() == 0 {
                    candidates.get_mut(&node).unwrap().insert(cand);
                }
            }
        }

        if candidates.get(&node).unwrap().is_empty() {
            // early exit: a node doesn't have any candidates, so no match can be found
            return Ok(vec![]);
        }
    }

    println!("CANDIDATES: {:?}", candidates);

    let keys = {
        let mut keys = candidates.keys().cloned().collect::<Vec<_>>();
        keys.sort_by(|k, v| {
            candidates
                .get(&k)
                .unwrap()
                .len()
                .cmp(&candidates.get(&v).unwrap().len())
        });
        VecDeque::from_iter(keys)
    };

    let mut matchings_by_node = MultiSet::default();
    let mut edge_graph = MultiSet::default();
    let mut re_edge_graph = MultiSet::default(); // needed for easy (+3)

    for key in &keys {
        for neighbor in pattern_graph.get_front_neighbors(*key) {
            for key_tgt in candidates.get(key).unwrap() {
                for tgt_neighbor in target_children.get_front_neighbors(*key_tgt) {
                    let s1t1 = (*key, *key_tgt);
                    let s2t2 = (neighbor, tgt_neighbor);
                    if candidates.get(&neighbor).unwrap().contains(&tgt_neighbor) {
                        matchings_by_node.insert(*key, s1t1);
                        edge_graph.insert(s1t1, s2t2);
                        re_edge_graph.insert(s2t2, s1t1);
                    }
                }
            }
        }
    }

    let mut pairing_to_index = HashMap::<((u32, u32), (u32, u32)), u32>::new();
    let mut index_to_pairing = HashMap::<u32, ((u32, u32), (u32, u32))>::new();
    let mut perpendicularity = UndirectedAdjacencyMatrix::default();

    let mut graph_dot = vec![];
    graph_dot.push(format!("graph G {{"));
    for (s1t1, s2t2s) in &edge_graph.set {
        for s2t2 in s2t2s {
            let key = (s1t1.clone(), s2t2.clone());
            let index = perpendicularity.adjacency.len() as u32;
            //println!("KEY: {:?} -> INDEX: {}", key, index);
            perpendicularity.add_node(index);
            pairing_to_index.insert(key, index);
            index_to_pairing.insert(index, key);

            graph_dot.push(format!("  a{} [label=\"{:?}, {:?}\"]", index, *s1t1, *s2t2));
        }
    }

    for ((s1t1, s2t2), index) in pairing_to_index.clone() {
        /*  +1:
             S1: T1  -> <S2: T2>
            <S2: T2> ->  S3: T3
        */
        if let Some(s3t3s) = edge_graph.set.get(&s2t2).cloned() {
            for s3t3 in s3t3s {
                if tuple_is_completely_different(&s1t1, &s3t3) {
                    let second_index = *pairing_to_index.get(&(s2t2, s3t3)).unwrap();

                    let foreign_key1 = (s1t1, s3t3);
                    let foreign_key2 = (s3t3, s1t1);
                    if pairing_to_index.contains_key(&foreign_key1)
                        || pairing_to_index.contains_key(&foreign_key2)
                    {
                        if !perpendicularity.are_adjacent(index, second_index) {
                            perpendicularity.add_edge(arrow_counters, index, second_index);
                            arrow_counters -= 1;

                            graph_dot.push(format!(
                                "  a{} -- a{} [label=\"{}\"]",
                                index, second_index, "(+1)"
                            ));
                        }
                    }
                }
            }
        }

        /*  +2:
             <S1: T1> -> S2: T2
             <S1: T1> -> S3: T3
        */
        if let Some(s3t3s) = edge_graph.set.get(&s1t1).cloned() {
            for s3t3 in s3t3s {
                if tuple_is_completely_different(&s2t2, &s3t3) {
                    let second_index = *pairing_to_index.get(&(s1t1, s3t3)).unwrap();

                    let foreign_key1 = (s2t2, s3t3);
                    let foreign_key2 = (s3t3, s2t2);
                    if pairing_to_index.contains_key(&foreign_key1)
                        || pairing_to_index.contains_key(&foreign_key2)
                    {
                        if !perpendicularity.are_adjacent(index, second_index) {
                            perpendicularity.add_edge(arrow_counters, index, second_index);
                            arrow_counters -= 1;

                            graph_dot.push(format!(
                                "  a{} -- a{} [label=\"{}\"]",
                                index, second_index, "(+2)"
                            ));
                        }
                    }
                }
            }
        }

        /*  +3:
             S1: T1 -> <S2: T2>
             S3: T3 -> <S2: T2>
        */
        if let Some(s3t3s) = re_edge_graph.set.get(&s2t2).cloned() {
            for s3t3 in s3t3s {
                if tuple_is_completely_different(&s1t1, &s3t3) {
                    let second_index = *pairing_to_index.get(&(s3t3, s2t2)).unwrap();

                    let foreign_key1 = (s1t1, s3t3);
                    let foreign_key2 = (s3t3, s1t1);
                    if pairing_to_index.contains_key(&foreign_key1)
                        || pairing_to_index.contains_key(&foreign_key2)
                    {
                        if !perpendicularity.are_adjacent(index, second_index) {
                            perpendicularity.add_edge(arrow_counters, index, second_index);
                            arrow_counters -= 1;

                            graph_dot.push(format!(
                                "  a{} -- a{} [label=\"{}\"]",
                                index, second_index, "(+3)"
                            ));
                        }
                    }
                }
            }
        }
    }

    graph_dot.push(format!("}}"));

    {
        //let mut file = File::create("./graph.dot").unwrap();
        //file.write(graph_dot.join("\n").as_bytes()).unwrap();
        //println!("------------------------------------------------------ DOTFILE SAVED");
    }

    let mut maps = HashSet::new();

    for node in perpendicularity.get_all_nodes() {
        for path in perpendicularity.dfs(node) {
            if path.len() == pattern_size - 1 {
                println!("PATH: {:?}", path);

                let mut bindings = HashMap::new();
                let mut values = HashSet::new();

                //println!("{}", path.iter().map(|p| format!("{:?}", index_to_pairing.get(p).unwrap())).collect::<Vec<_>>().join(" --> "));
                for step in path {
                    let ((s1, t1), (s2, t2)) = index_to_pairing.get(&step).unwrap().clone();
                    values.insert(t1);
                    values.insert(t2);
                    if !bindings.contains_key(&s1) {
                        bindings.insert(s1, t1);
                    }
                    if !bindings.contains_key(&s2) {
                        bindings.insert(s2, t2);
                    }
                }

                if values.len() == pattern_size && bindings.len() == pattern_size {
                    let h = hash_hashmap(&bindings);

                    if !maps.contains(&h) {
                        let binding_frame = engine_state.new_node(arrow.entity)?;
                        for (k, v) in bindings {
                            new_arrow(binding_frame, k, v)?;
                        }
                        maps.insert(h);
                    }
                }
            }
        }
    }

    let mut result_block = vec![];
    result_block.push(*arrow);

    Ok(result_block)
}

pub struct GraphMatchHelper {
    block: Block,
}

pub fn run_graph_match(block: &Block) -> Result<GraphMatchHelper, String> {
    let res = graph_match(block)?;

    Ok(GraphMatchHelper { block: res })
}

impl GraphMatchHelper {
    pub fn solution_count(&self) -> usize {
        self.block.get_bricks().len()
    }

    pub fn get_solution(&self, index: usize) -> HashMap<EntityId, EntityId> {
        let mut binding = HashMap::new();

        if let Some(solution) =
            get_framed_entities(self.block.get_bricks().first().unwrap().entity).get(index)
        {
            for arrow in get_framed_entities(*solution) {
                let src = get_comp_field_u32(arrow, "Arrow", "Arrow.source").unwrap();
                let tgt = get_comp_field_u32(arrow, "Arrow", "Arrow.target").unwrap();
                binding.insert(src, tgt);
            }
        }
        binding
    }

    pub fn is_empty(&self) -> bool {
        self.block.get_bricks().len() == 0
    }
}

impl Drop for GraphMatchHelper {
    fn drop(&mut self) {
        if let Some(frame) = self.block.get_bricks().first() {
            if is_entity_valid(frame.entity) {
                for node in get_framed_entities(frame.entity) {
                    delete_node(frame.entity, node);
                }
            }
        }
    }
}

#[cfg(test)]
mod search_tests {
    use crate::{
        backend_plugins::graph_match::graph_match,
        ec::add_component,
        graph::get_graph_refs,
        protocol::{v1::register_type, *},
        top_level::*,
    };

    #[test]
    fn test_search_plus1and2() {
        initialize_engine();

        let label_type = Type::Pure(PureType {
            name: component_name("Label"),
            datatype: DataType::NAME,
        });
        register_type(&label_type);

        let graph_match_type = Type::Pure(PureType {
            name: component_name("GraphMatch"),
            datatype: DataType::VOID,
        });
        register_type(&graph_match_type);

        let pattern = new_node(0).unwrap(); // 1
        {
            let a = new_node(pattern).unwrap(); // a 2
            let b = new_node(pattern).unwrap(); // b 3
            let c = new_node(pattern).unwrap(); // c 4
            let d = new_node(pattern).unwrap(); // d 5
            new_arrow(pattern, a, b).unwrap(); // a -> b 6
            new_arrow(pattern, a, c).unwrap(); // a -> c 7
            new_arrow(pattern, c, d).unwrap(); // c -> d 8
        }

        let target = new_node(0).unwrap(); // 9
        {
            let x = new_node(target).unwrap(); // x 10
            let y = new_node(target).unwrap(); // y 11
            let z = new_node(target).unwrap(); // z 12
            let h = new_node(target).unwrap(); // h 13
            new_arrow(target, x, y).unwrap(); // x -> y 14
            new_arrow(target, y, z).unwrap(); // y -> z 15
            new_arrow(target, y, h).unwrap(); // y -> h 16
            new_arrow(target, h, y).unwrap(); // h -> y 17
            new_arrow(target, h, x).unwrap(); // h -> x 18
        }
        // 2 -> 13, 4 -> 10, 5 -> 11, 3 -> 12
        // a -> h, b -> x, c -> y, d -> z
        // a -> y, b -> z, c -> h, d -> x
        let arrow = new_arrow(0, pattern, target).unwrap();
        add_component(arrow, &graph_match_type);

        let mut block = Block::default();
        block.extend_with(entity_to_block_no_children(arrow).unwrap());

        println!("{:?}", block);

        let result = graph_match(&block).unwrap();
        println!("{:?}", result);

        assert!(result.get_bricks().first().is_some());
        let arrow_id = result.get_bricks().first().unwrap().entity;
        assert!(arrow == arrow_id);

        let bindings = get_graph_refs().read().unwrap().get_vec(&arrow_id).cloned();
        assert!(bindings.is_none());
    }

    #[test]
    fn test_search_worst_case() {
        initialize_engine();

        let label_type = Type::Pure(PureType {
            name: component_name("Label"),
            datatype: DataType::NAME,
        });
        register_type(&label_type);

        let graph_match_type = Type::Pure(PureType {
            name: component_name("GraphMatch"),
            datatype: DataType::VOID,
        });
        register_type(&graph_match_type);

        let pattern = new_node(0).unwrap(); // 1
        {
            let a = new_node(pattern).unwrap(); // a 2
            let b = new_node(pattern).unwrap(); // b 3
            let c = new_node(pattern).unwrap(); // c 4
            let d = new_node(pattern).unwrap(); // d 5
            let e = new_node(pattern).unwrap(); // d 5
            new_arrow(pattern, a, b).unwrap();
            new_arrow(pattern, a, c).unwrap();
            new_arrow(pattern, a, d).unwrap();
            new_arrow(pattern, a, e).unwrap();
            new_arrow(pattern, b, a).unwrap();
            new_arrow(pattern, b, c).unwrap();
            new_arrow(pattern, b, d).unwrap();
            new_arrow(pattern, b, e).unwrap();
            new_arrow(pattern, c, a).unwrap();
            new_arrow(pattern, c, b).unwrap();
            new_arrow(pattern, c, d).unwrap();
            new_arrow(pattern, c, e).unwrap();
            new_arrow(pattern, d, a).unwrap();
            new_arrow(pattern, d, b).unwrap();
            new_arrow(pattern, d, c).unwrap();
            new_arrow(pattern, d, e).unwrap();
            new_arrow(pattern, e, a).unwrap();
            new_arrow(pattern, e, b).unwrap();
            new_arrow(pattern, e, c).unwrap();
            new_arrow(pattern, e, d).unwrap();
        }

        let target = new_node(0).unwrap(); // 9
        {
            let a = new_node(target).unwrap(); // a 2
            let b = new_node(target).unwrap(); // b 3
            let c = new_node(target).unwrap(); // c 4
            let d = new_node(target).unwrap(); // d 5
            let e = new_node(target).unwrap(); // d 5
            new_arrow(target, a, b).unwrap();
            new_arrow(target, a, c).unwrap();
            new_arrow(target, a, d).unwrap();
            new_arrow(target, a, e).unwrap();
            new_arrow(target, b, a).unwrap();
            new_arrow(target, b, c).unwrap();
            new_arrow(target, b, d).unwrap();
            new_arrow(target, b, e).unwrap();
            new_arrow(target, c, a).unwrap();
            new_arrow(target, c, b).unwrap();
            new_arrow(target, c, d).unwrap();
            new_arrow(target, c, e).unwrap();
            new_arrow(target, d, a).unwrap();
            new_arrow(target, d, b).unwrap();
            new_arrow(target, d, c).unwrap();
            new_arrow(target, d, e).unwrap();
            new_arrow(target, e, a).unwrap();
            new_arrow(target, e, b).unwrap();
            new_arrow(target, e, c).unwrap();
            new_arrow(target, e, d).unwrap();
        }

        let arrow = new_arrow(0, pattern, target).unwrap();
        add_component(arrow, &graph_match_type);

        let mut block = Block::default();
        block.extend_with(entity_to_block_no_children(arrow).unwrap());

        println!("{:?}", block);

        let result = graph_match(&block).unwrap();
        println!("{:?}", result);

        assert!(result.get_bricks().first().is_some());
        let arrow_id = result.get_bricks().first().unwrap().entity;
        assert!(arrow == arrow_id);

        let bindings = get_graph_refs().read().unwrap().get_vec(&arrow_id).cloned();
        assert!(bindings.is_some());

        let bindings = bindings.unwrap();
        //        assert_eq!(bindings.len(), 1);

        println!("{:?}", bindings);
        for b in bindings {
            println!("{:?}", get_graph_for_entity(b));
        }
    }
}
