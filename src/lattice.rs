use kanpyo_dict::{dict::Dict, morph::Morph, trie::da::KeywordID};
use node::Node;
pub mod node;
// Lattice represents a grid of morph nodes.
#[derive(Debug, Clone)]
pub struct Lattice<'a> {
    pub dict: &'a Dict,
    pub nodes: Vec<node::Node>,
    pub edges: Vec<Vec<usize>>,
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

    /// Searches for known words in the dictionary at the given `byte_pos`.
    /// Returns `true` if at least one match is found, otherwise `false`.
    fn process_known_words(&mut self, byte_pos: usize, char_pos: usize, input: &str) -> bool {
        let text = &input[byte_pos..];
        if let Some(ids_and_byte_lengths) = self.dict.index_table.search_common_prefix_of(text) {
            for (id, byte_length) in ids_and_byte_lengths {
                let end_byte_pos = byte_pos + byte_length;
                let surface = &input[byte_pos..end_byte_pos];
                self.add_known_node(id, byte_pos, char_pos, surface);
            }
            true
        } else {
            false
        }
    }

    /// Processes unknown words if needed. This is triggered either if no known words matched
    /// or if the dictionary's `invoke_list` requires unknown processing for this character category.
    fn process_unknown_words(
        &mut self,
        byte_pos: usize,
        char_pos: usize,
        ch: char,
        input: &str,
        matched_known: bool,
    ) {
        // Determine the character category of `ch`.
        let char_category = self.dict.char_category_def.char_category(ch);

        // If no known matches or if the dictionary explicitly requires unknown processing, proceed.
        if !matched_known || self.dict.char_category_def.invoke_list[char_category as usize] {
            const MAXIMUM_UNKNOWN_WORD_LENGTH: usize = 1024;

            // If this category supports grouping, we can bundle consecutive characters of the same type.
            let is_group = *self
                .dict
                .char_category_def
                .group_list
                .get(char_category as usize)
                .unwrap_or(&false);

            // Determine how far the unknown sequence goes.
            let mut end_byte_pos = byte_pos + ch.len_utf8();
            let mut unknown_word_length = 1;

            // Extend the unknown sequence while the next character is in the same category.
            if is_group {
                for next_char in input[end_byte_pos..].chars() {
                    let next_category = self.dict.char_category_def.char_category(next_char);
                    if next_category != char_category {
                        break;
                    }
                    end_byte_pos += next_char.len_utf8();
                    unknown_word_length += 1;

                    // Limit the maximum length of unknown sequences.
                    if unknown_word_length >= MAXIMUM_UNKNOWN_WORD_LENGTH {
                        break;
                    }
                }
            }

            // Get the registered (morph_id, count) for this character category in the unknown dictionary.
            if let Some(&(morph_id, count)) = self
                .dict
                .unk_dict
                .char_category_to_morph_id
                .get(&char_category)
            {
                let surface = &input[byte_pos..end_byte_pos];
                for i in 0..count {
                    self.add_unknown_node(morph_id + i as isize, byte_pos, char_pos, surface);
                }
            }
        }
    }

    pub fn build(dict: &'a Dict, input: &str) -> Self {
        let mut byte_pos = 0;
        let mut la = Lattice::new(dict, input);
        la.add_bos_node();
        for (char_pos, ch) in input.chars().enumerate() {
            // Known words
            let matched_known = la.process_known_words(byte_pos, char_pos, input);
            // Unknown words
            la.process_unknown_words(byte_pos, char_pos, ch, input, matched_known);
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
                    dp[i].is_none_or(|c| total_cost < c).then(|| {
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
