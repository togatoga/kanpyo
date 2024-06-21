use std::collections::{BTreeMap, BTreeSet};

use kanpyo_dict::{dict::Dict, morph::Morph, trie::da::KeywordID};
use node::Node;
pub mod node;
// Lattice represents a grid of morph nodes.
#[derive(Debug, Clone)]
pub struct Lattice<'a> {
    dict: &'a Dict,
    nodes: Vec<node::Node>,
    edges: Vec<Vec<usize>>,
}

impl<'a> Lattice<'a> {
    fn new(dict: &'a Dict, input: &str) -> Self {
        let edges = vec![vec![]; input.chars().count() + 2];
        Lattice {
            dict,
            nodes: vec![],
            edges,
        }
    }

    pub fn build(dict: &'a Dict, input: &str) -> Self {
        let mut byte_pos = 0;
        let mut la = Lattice::new(dict, input);
        la.add_bos_node();
        const MAXIMUM_UNKNOWN_WORD_LENGTH: usize = 1024;
        for (char_pos, ch) in input.chars().enumerate() {
            let mut any_match = false;
            // Known words
            let text = input.get(byte_pos..).expect("byte_pos is out of range");
            if let Some(ids_and_byte_lenghts) = dict.index_table.search_common_prefix_of(text) {
                any_match = true;
                for (id, byte_length) in ids_and_byte_lenghts {
                    let surface = input
                        .get(byte_pos..byte_pos + byte_length)
                        .expect("byte_pos is out of range");
                    la.add_known_node(id, byte_pos, char_pos, surface);
                    // la.add_node(id, byte_pos, char_pos, NodeClass::Known, Some(surface));
                }
            }
            let char_category = dict.char_category_def.char_category(ch);
            if !any_match || dict.char_category_def.invoke_list[char_category as usize] {
                // Unknown words
                let is_group = *dict
                    .char_category_def
                    .group_list
                    .get(char_category as usize)
                    .unwrap_or(&false);
                let mut end_byte_pos = byte_pos + ch.len_utf8();
                let mut unknown_word_length = 1;
                if is_group {
                    for ch in input[end_byte_pos..].chars() {
                        if dict.char_category_def.char_category(ch) != char_category {
                            break;
                        }
                        end_byte_pos += ch.len_utf8();
                        unknown_word_length += 1;
                        if unknown_word_length >= MAXIMUM_UNKNOWN_WORD_LENGTH {
                            break;
                        }
                    }
                }

                if let Some(&(morph_id, count)) =
                    dict.unk_dict.char_category_to_morph_id.get(&char_category)
                {
                    let surface = input
                        .get(byte_pos..end_byte_pos)
                        .expect("byte_pos is out of range");
                    for i in 0..count {
                        la.add_unknown_node(morph_id + i as isize, byte_pos, char_pos, surface);
                    }
                }
            }
            byte_pos += ch.len_utf8();
        }
        la.add_eos_node(input);
        la
    }
    // runs forward algorithm of the Viterbi.
    pub fn viterbi(&self) -> Vec<Node> {
        const INF: i32 = 1 << 30;
        let mut dp = vec![None; self.nodes.len()];
        let mut pre_nodes = vec![None; self.nodes.len()];
        let char_len = self.edges.len();
        for char_pos in 1..char_len {
            for (i, target) in self.edges[char_pos].iter().map(|&i| (i, &self.nodes[i])) {
                dp[i] = Some(INF);
                let char_pos = target.char_pos();
                for (j, previous) in self.edges[char_pos].iter().map(|&i| (i, &self.nodes[i])) {
                    // let cost = target.morph.as_ref().map_or(0, |m| m.cost) as i32;
                    let prev_cost = dp[j].unwrap_or(0);
                    let previous_morph = previous.morph();
                    let target_morph = target.morph();
                    let cost = target_morph.cost as i32;
                    let matrix_cost = self.dict.connection_table.get(
                        previous_morph.right_id as usize,
                        target_morph.left_id as usize,
                    ) as i32;
                    let total_cost = (prev_cost + cost + matrix_cost).min(INF);
                    dp[i].map_or(true, |c| total_cost < c).then(|| {
                        dp[i] = Some(total_cost);
                        pre_nodes[i] = Some(j);
                    });
                }
            }
        }

        let mut pos: usize = self.nodes.len() - 1;
        let mut paths = vec![];
        while let Some(pre) = pre_nodes[pos] {
            let node = self.nodes[pos].clone();
            paths.push(node);
            pos = pre;
        }
        paths.reverse();
        paths
    }

    pub fn graphviz(&self, dpi: usize, unk: bool) {
        let bests = self.viterbi().into_iter().collect::<BTreeSet<Node>>();
        println!("graph lattice {{");
        println!("dpi={dpi};");
        println!("graph [style=filled, splines=true, overlap=false, fontsize=30, rankdir=LR]");
        println!("edge [fontname=Helvetica, fontcolor=red, color=\"#606060\"]");
        println!("node [shape=box, style=filled, fillcolor=\"#e8e8f0\", fontname=Helvetica]");
        let mut node_to_id = BTreeMap::default();
        for (id, node) in self.nodes.iter().enumerate() {
            let is_unk = matches!(node, Node::Unknown(_));
            if !unk && is_unk && !bests.contains(node) {
                continue;
            }
            node_to_id.insert(node, id);

            let label = match node {
                Node::Known(node) => format!(
                    "{}\n{}\n{}",
                    node.surface,
                    self.dict.morph_feature_table.morph_features[node.id as usize - 1]
                        .iter()
                        .map(|&idx| self.dict.morph_feature_table.name_list[idx as usize].clone())
                        .filter(|s| s != "*")
                        .collect::<Vec<String>>()
                        .join("/"),
                    node.morph.cost
                ),
                Node::Unknown(node) => format!(
                    "{}\n{}\n{}",
                    node.surface,
                    self.dict.unk_dict.morph_feature_table.morph_features[node.id as usize - 1]
                        .iter()
                        .map(
                            |&idx| self.dict.unk_dict.morph_feature_table.name_list[idx as usize]
                                .clone()
                        )
                        .filter(|s| s != "*")
                        .collect::<Vec<String>>()
                        .join("/"),
                    node.morph.cost
                ),
                _ => {
                    if id == 0 {
                        "BOS".to_string()
                    } else if id == self.nodes.len() - 1 {
                        "EOS".to_string()
                    } else {
                        "".to_string()
                    }
                }
            };

            if id == 0 || id == self.nodes.len() - 1 || bests.contains(node) {
                println!("{} [label=\"{}\", shape=ellipse, peripheries=2]", id, label);
            } else {
                if !unk && is_unk {
                    continue;
                }
                let shape = match node {
                    Node::Known(_) => "box",
                    Node::Unknown(_) => "diamond",
                    Node::Dummy { .. } => "ellipse",
                };
                println!("{} [label=\"{}\", shape={}]", id, label, shape);
                // println!("{} [label=\"{}\"]", id, label);
            }
        }

        for edge in self.edges.iter() {
            for node in edge.iter().map(|i| &self.nodes[*i]) {
                let to_id = node_to_id.get(node);
                if to_id.is_none() {
                    continue;
                }
                let to_id = to_id.unwrap();
                for &from in self.edges[node.char_pos()].iter() {
                    let from_node = &self.nodes[from];
                    let from_id = node_to_id.get(from_node);
                    if from_id.is_none() {
                        continue;
                    }
                    let from_id = from_id.unwrap();
                    if from_id == to_id {
                        continue;
                    }

                    let label = format!(
                        "{}",
                        self.dict.connection_table.get(
                            from_node.morph().right_id as usize,
                            node.morph().left_id as usize
                        )
                    );
                    let ok1 = bests.contains(from_node) || *from_id == 0;
                    let ok2 = bests.contains(node) || *to_id == self.nodes.len() - 1;
                    if ok1 && ok2 {
                        println!(
                            "{} -- {} [label=\"{}\", style=bold, color=blue, fontcolor=blue]",
                            from_id, to_id, label
                        );
                    } else {
                        println!("{} -- {} [label=\"{}\"]", from_id, to_id, label);
                    }
                }
            }
        }
        println!("}}");
    }

    fn add_bos_node(&mut self) {
        let idx = self.nodes.len();
        self.nodes.push(node::Node::Dummy {
            byte_pos: 0,
            char_pos: 0,
            morph: Morph::new(0, 0, 0),
        });
        self.edges[0].push(idx);
    }
    fn add_eos_node(&mut self, input: &str) {
        let idx = self.nodes.len();
        let byte_pos = input.len();
        let char_pos = input.chars().count();
        self.nodes.push(node::Node::Dummy {
            byte_pos,
            char_pos,
            morph: Morph::new(0, 0, 0),
        });
        self.edges[char_pos + 1].push(idx);
    }

    fn add_known_node(&mut self, id: KeywordID, byte_pos: usize, char_pos: usize, surface: &str) {
        let node = node::Word {
            id,
            byte_pos,
            char_pos,
            morph: self.dict.morphs[id - 1].clone(),
            surface: surface.to_string(),
        };
        let idx = self.nodes.len();
        self.nodes.push(node::Node::Known(node));
        self.edges[char_pos + surface.chars().count()].push(idx);
    }

    fn add_unknown_node(&mut self, id: KeywordID, byte_pos: usize, char_pos: usize, surface: &str) {
        let node = node::Word {
            id,
            byte_pos,
            char_pos,
            morph: self.dict.unk_dict.morphs[id - 1].clone(),
            surface: surface.to_string(),
        };
        let idx = self.nodes.len();
        self.nodes.push(node::Node::Unknown(node));
        self.edges[char_pos + surface.chars().count()].push(idx);
    }
}
