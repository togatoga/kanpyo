use crate::token::TokenClass;
use crate::tokenizer::Tokenizer;
use kanpyo_dict::builder::matrix_def::MatrixDef;
use kanpyo_dict::dict::Dict;
use kanpyo_dict::{char_category_def, connection, index, morph, morph_feature, unk_dict};

/// Creates a minimal test dictionary for testing purposes
fn create_test_dict() -> Dict {
    let sorted_keywords = vec![
        "テスト".to_string(),
        "辞書".to_string(),
        "形態素".to_string(),
    ];
    let index = index::IndexTable::build(&sorted_keywords).expect("Failed to build index table");

    let morphs = morph::Morphs::from(vec![
        morph::Morph::new(0, 0, 1000),
        morph::Morph::new(1, 1, 1200),
        morph::Morph::new(2, 2, 1100),
    ]);

    let morph_feature_table = morph_feature::MorphFeatureTableBuilder::from(vec![
        vec![
            "名詞",
            "一般",
            "*",
            "*",
            "*",
            "*",
            "テスト",
            "テスト",
            "テスト",
        ],
        vec![
            "名詞",
            "一般",
            "*",
            "*",
            "*",
            "*",
            "辞書",
            "ジショ",
            "ジショ",
        ],
        vec![
            "名詞",
            "一般",
            "*",
            "*",
            "*",
            "*",
            "形態素",
            "ケイタイソ",
            "ケイタイソ",
        ],
    ])
    .build();

    let connection_table = connection::ConnectionTable::from(MatrixDef {
        row: 3,
        col: 3,
        data: vec![0, 100, 200, 100, 0, 100, 200, 100, 0],
    });

    let char_category_def = char_category_def::CharCategoryDef {
        char_class: vec![
            "DEFAULT".to_string(),
            "KANJI".to_string(),
            "HIRAGANA".to_string(),
        ],
        char_category: {
            let mut cat = vec![0u8; 65536];
            // Set Japanese characters to HIRAGANA category
            for ch in 'あ'..='ん' {
                cat[ch as usize] = 2;
            }
            // Set Kanji to KANJI category (simplified)
            for ch in '一'..='龥' {
                cat[ch as usize] = 1;
            }
            cat
        },
        invoke_list: vec![false, true, true],
        group_list: vec![false, true, true],
    };

    let unk_dict = unk_dict::UnkDict {
        morphs: morph::Morphs::from(vec![
            morph::Morph::new(0, 0, 5000),
            morph::Morph::new(1, 1, 5000),
        ]),
        morph_feature_table: morph_feature::MorphFeatureTableBuilder::from(vec![
            vec!["未知語", "*", "*", "*", "*", "*", "*", "*", "*"],
            vec!["未知語", "*", "*", "*", "*", "*", "*", "*", "*"],
        ])
        .build(),
        char_category_to_morph_id: vec![(1, (1, 1)), (2, (2, 1))].into_iter().collect(),
    };

    Dict::new(
        morphs,
        morph_feature_table,
        connection_table,
        index,
        char_category_def,
        unk_dict,
    )
}

#[test]
fn test_tokenizer_basic() {
    let dict = create_test_dict();
    let tokenizer = Tokenizer::new(dict);

    // Test with known words
    let tokens = tokenizer.tokenize("テスト");
    assert!(!tokens.is_empty(), "Should tokenize known word");

    // Check that we have at least one actual token (not just EOS)
    let non_eos_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.class != TokenClass::Dummy)
        .collect();
    assert!(
        !non_eos_tokens.is_empty(),
        "Should have at least one non-EOS token"
    );
}

#[test]
fn test_tokenizer_empty_input() {
    let dict = create_test_dict();
    let tokenizer = Tokenizer::new(dict);

    let tokens = tokenizer.tokenize("");
    // Empty input should still produce EOS token
    assert!(
        !tokens.is_empty(),
        "Should produce EOS token for empty input"
    );
}

#[test]
fn test_tokenizer_unknown_word() {
    let dict = create_test_dict();
    let tokenizer = Tokenizer::new(dict);

    let tokens = tokenizer.tokenize("あいうえお");
    // Should be able to tokenize unknown words using unknown word processing
    assert!(!tokens.is_empty(), "Should tokenize unknown words");
}

#[test]
fn test_token_positions() {
    let dict = create_test_dict();
    let tokenizer = Tokenizer::new(dict);

    let input = "テスト";
    let tokens = tokenizer.tokenize(input);

    for token in &tokens {
        if token.class != TokenClass::Dummy {
            // Check that positions are within bounds
            assert!(token.start <= token.end, "Token start should be <= end");
            assert!(
                token.end <= input.chars().count(),
                "Token end should be within input"
            );
        }
    }
}

#[test]
fn test_tokenizer_dict_roundtrip() {
    let dict = create_test_dict();

    // Serialize and deserialize the dictionary
    let mut buffer = std::io::Cursor::new(Vec::new());
    dict.build(&mut buffer).expect("Failed to build dict");

    buffer.set_position(0);
    let loaded_dict = Dict::load(&mut buffer).expect("Failed to load dict");

    // Create tokenizers with both dicts
    let tokenizer1 = Tokenizer::new(dict);
    let tokenizer2 = Tokenizer::new(loaded_dict);

    // They should produce the same results
    let tokens1 = tokenizer1.tokenize("テスト");
    let tokens2 = tokenizer2.tokenize("テスト");

    assert_eq!(
        tokens1.len(),
        tokens2.len(),
        "Should produce same number of tokens"
    );

    for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
        assert_eq!(t1.surface, t2.surface, "Tokens should have same surface");
        assert_eq!(t1.class, t2.class, "Tokens should have same class");
    }
}
