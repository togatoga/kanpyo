use dict::{dict::Dict, trie::da::KeywordID};
use node::{Node, NodeClass};
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
                    la.add_node(id, byte_pos, char_pos, NodeClass::Known, Some(surface));
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
                let surface = input.get(byte_pos..end_byte_pos);
                if let Some(&(morph_id, count)) =
                    dict.unk_dict.char_category_to_morph_id.get(&char_category)
                {
                    for i in 0..count {
                        la.add_node(
                            morph_id + i as isize,
                            byte_pos,
                            char_pos,
                            NodeClass::Unknown,
                            surface,
                        );
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
                for (j, previous) in self.edges[target.char_pos]
                    .iter()
                    .map(|&i| (i, &self.nodes[i]))
                {
                    let cost = target.morph.as_ref().map_or(0, |m| m.cost) as i32;
                    let prev_cost = dp[j].unwrap_or(0);
                    let matrix_cost = match (previous.morph.as_ref(), target.morph.as_ref()) {
                        (Some(prev_morph), Some(target_morph)) => self
                            .dict
                            .connection_table
                            .get(prev_morph.right_id as usize, target_morph.left_id as usize)
                            as i32,
                        _ => 0,
                    };
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

    fn add_bos_node(&mut self) {
        self.add_node(node::BOS_EOS_ID, 0, 0, NodeClass::Dummy, None);
    }
    fn add_eos_node(&mut self, input: &str) {
        let char_pos = input.chars().count();
        let node: Node = node::Node {
            id: node::BOS_EOS_ID,
            byte_pos: input.len(),
            char_pos,
            class: NodeClass::Dummy,
            morph: None,
            surface: None,
        };
        let idx = self.nodes.len();
        self.nodes.push(node);
        self.edges[char_pos + 1].push(idx);
    }
    fn add_node(
        &mut self,
        id: KeywordID,
        byte_pos: usize,
        char_pos: usize,
        class: NodeClass,
        surface: Option<&str>,
    ) {
        let morph = match class {
            NodeClass::Known => Some(self.dict.morphs[id - 1].clone()),
            NodeClass::Unknown => Some(self.dict.unk_dict.morphs[id - 1].clone()),
            _ => None,
        };
        let node = node::Node {
            id,
            byte_pos,
            char_pos,
            class,
            morph,
            surface: surface.map(|s| s.to_string()),
        };
        let idx = self.nodes.len();
        self.nodes.push(node);
        let p: usize = char_pos + surface.map_or(0, |s| s.chars().count());
        self.edges[p].push(idx);
    }
}
