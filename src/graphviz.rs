use std::collections::{BTreeMap, BTreeSet};

use crate::lattice::{node::Node, Lattice};

pub struct Graphviz<'a> {
    pub lattice: Lattice<'a>,
}

impl<'a> Graphviz<'a> {
    pub fn graphviz(&self, dpi: usize, unk: bool) {
        let bests = self
            .lattice
            .viterbi()
            .into_iter()
            .collect::<BTreeSet<Node>>();
        println!("graph lattice {{");
        println!("dpi={dpi};");
        println!("graph [style=filled, splines=true, overlap=false, fontsize=30, rankdir=LR]");
        println!("edge [fontname=Helvetica, fontcolor=red, color=\"#606060\"]");
        println!("node [shape=box, style=filled, fillcolor=\"#e8e8f0\", fontname=Helvetica]");
        let mut node_to_id = BTreeMap::default();
        for (id, node) in self.lattice.nodes.iter().enumerate() {
            let is_unk = matches!(node, Node::Unknown(_));
            if !unk && is_unk && !bests.contains(node) {
                continue;
            }
            node_to_id.insert(node, id);

            let label = match node {
                Node::Known(node) => format!(
                    "{}\n{}\n{}",
                    node.surface,
                    self.lattice.dict.morph_feature_table.morph_features[node.id as usize - 1]
                        .iter()
                        .map(
                            |&idx| self.lattice.dict.morph_feature_table.name_list[idx as usize]
                                .clone()
                        )
                        .filter(|s| s != "*")
                        .collect::<Vec<String>>()
                        .join("/"),
                    node.morph.cost
                ),
                Node::Unknown(node) => format!(
                    "{}\n{}\n{}",
                    node.surface,
                    self.lattice
                        .dict
                        .unk_dict
                        .morph_feature_table
                        .morph_features[node.id as usize - 1]
                        .iter()
                        .map(
                            |&idx| self.lattice.dict.unk_dict.morph_feature_table.name_list
                                [idx as usize]
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
                    } else if id == self.lattice.nodes.len() - 1 {
                        "EOS".to_string()
                    } else {
                        "".to_string()
                    }
                }
            };

            if id == 0 || id == self.lattice.nodes.len() - 1 || bests.contains(node) {
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

        for edge in self.lattice.edges.iter() {
            for node in edge.iter().map(|i| &self.lattice.nodes[*i]) {
                let to_id = node_to_id.get(node);
                if to_id.is_none() {
                    continue;
                }
                let to_id = to_id.unwrap();
                for &from in self.lattice.edges[node.char_pos()].iter() {
                    let from_node = &self.lattice.nodes[from];
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
                        self.lattice.dict.connection_table.get(
                            from_node.morph().right_id as usize,
                            node.morph().left_id as usize
                        )
                    );
                    let ok1 = bests.contains(from_node) || *from_id == 0;
                    let ok2 = bests.contains(node) || *to_id == self.lattice.nodes.len() - 1;
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
}
