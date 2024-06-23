use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::lattice::{node::Node, Lattice};

pub struct Graphviz<'a> {
    pub lattice: Lattice<'a>,
}

impl<'a> Graphviz<'a> {
    fn bfs(&self, start: Node, bests: &BTreeSet<Node>) -> Vec<Node> {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::default();
        queue.push_back(start);
        while let Some(node) = queue.pop_front() {
            if !visited.insert(node.clone()) {
                continue;
            }
            for node in self.lattice.edges[node.char_pos()]
                .iter()
                .map(|i| &self.lattice.nodes[*i])
                .filter(|node| !visited.contains(node))
                .filter(|node| !matches!(node, Node::Unknown(_)) || bests.contains(node))
            {
                queue.push_back(node.clone());
            }
        }
        visited.into_iter().collect()
    }

    pub fn graphviz(&self, dpi: usize, full_state: bool) {
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

        let visible_nodes = if !full_state {
            self.bfs(
                self.lattice
                    .nodes
                    .last()
                    .expect("last node not found")
                    .clone(),
                &bests,
            )
        } else {
            self.lattice.nodes.clone()
        };

        for (visible_id, visible_node) in visible_nodes.iter().enumerate() {
            let label = match visible_node {
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
                    if visible_id == 0 {
                        "BOS".to_string()
                    } else {
                        "EOS".to_string()
                    }
                }
            };
            let color = match visible_node {
                Node::Known(_) => "black",
                Node::Unknown(_) => "red",
                Node::Dummy { .. } => "blue",
            };
            if bests.contains(visible_node) || matches!(visible_node, Node::Dummy { .. }) {
                println!(
                    "{} [label=\"{}\", shape=ellipse, color={}, peripheries=2]",
                    visible_id, label, color
                );
            } else {
                let shape = match visible_node {
                    Node::Known(_) => "box",
                    Node::Unknown(_) => "diamond",
                    Node::Dummy { .. } => "ellipse",
                };
                println!(
                    "{} [label=\"{}\", shape={}, color={}]",
                    visible_id, label, shape, color
                );
            }
        }
        let node_to_visible_id = visible_nodes
            .iter()
            .enumerate()
            .map(|(id, node)| (node, id))
            .collect::<BTreeMap<_, _>>();
        for edge in self.lattice.edges.iter() {
            for (id, node) in edge.iter().filter_map(|i| {
                node_to_visible_id
                    .get(&self.lattice.nodes[*i])
                    .map(|&id| (id, &self.lattice.nodes[*i]))
            }) {
                for (from_id, from_node) in
                    self.lattice.edges[node.char_pos()].iter().filter_map(|i| {
                        node_to_visible_id
                            .get(&self.lattice.nodes[*i])
                            .map(|&id| (id, &self.lattice.nodes[*i]))
                    })
                {
                    if from_id == id {
                        continue;
                    }

                    let label = format!(
                        "{}",
                        self.lattice.dict.connection_table.get(
                            from_node.morph().right_id as usize,
                            node.morph().left_id as usize
                        )
                    );
                    let ok1 = bests.contains(from_node) || matches!(from_node, Node::Dummy { .. });
                    let ok2 = bests.contains(node) || matches!(node, Node::Dummy { .. });
                    if ok1 && ok2 {
                        println!(
                            "{} -- {} [label=\"{}\", style=bold, color=blue, fontcolor=blue]",
                            from_id, id, label
                        );
                    } else {
                        println!("{} -- {} [label=\"{}\"]", from_id, id, label);
                    }
                }
            }
        }
        println!("}}");
    }
}
