use crate::tokenizer::{FileTokens, Tokenizer};
use std::{collections::HashMap, path::Path, path::PathBuf};

#[test]
fn test_tokenizer() {
    let dir = Path::new("tests/fixtures");
    let tkn = Tokenizer::new();
    let wd = tkn.walkdir(dir).unwrap();

    let expected: HashMap<&str, Vec<&str>> = [
        (
            "tests/fixtures/dir1/sample.txt",
            vec!["outdoor", "action", "film", "in"],
        ),
        (
            "tests/fixtures/sample.md",
            vec!["rust", "is", "my", "favourite", "program", "language"],
        ),
        (
            "tests/fixtures/sample.rs",
            vec!["the", "big", "cat", "sanctuary"],
        ),
        (
            "tests/fixtures/dir1/s1.txt",
            vec!["hello", "this", "is", "test"],
        ),
        ("tests/fixtures/edge_cases/empty.txt", vec![]),
        ("tests/fixtures/edge_cases/whitespace_only.txt", vec![]),
        (
            "tests/fixtures/edge_cases/multiline.txt",
            vec!["hello", "world", "foo", "bar"],
        ),
    ]
    .into_iter()
    .collect();

    for res in wd {
        let ft = res.unwrap();
        let path = ft.path().to_str().unwrap().to_string();
        let exp_tokens = expected
            .get(path.as_str())
            .expect(&format!("unexpected file: {path}"));
        let actual: Vec<String> = ft.map(|t| t.unwrap().text).collect();
        assert_eq!(&actual, exp_tokens, "mismatch for {path}");
    }
}

fn tokens(path: &str) -> Vec<crate::tokenizer::Token> {
    FileTokens::new(PathBuf::from(path))
        .map(|r| r.unwrap())
        .collect()
}

#[test]
fn test_punctuation_splitting() {
    // "Hello! This is,test" — '!' and ',' are word breakers
    let ts = tokens("tests/fixtures/dir1/s1.txt");
    let texts: Vec<&str> = ts.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(texts, vec!["hello", "this", "is", "test"]);
}

#[test]
fn test_lowercasing() {
    // All tokens must be lowercased regardless of source casing
    let ts = tokens("tests/fixtures/dir1/s1.txt");
    for t in &ts {
        assert_eq!(
            t.text,
            t.text.to_lowercase(),
            "token '{}' is not lowercase",
            t.text
        );
    }
}

#[test]
fn test_token_char_positions() {
    // "Hello! This is,test"
    let ts = tokens("tests/fixtures/dir1/s1.txt");
    assert_eq!(ts.len(), 4);

    assert_eq!(ts[0].pos.char_off, 0);
    assert_eq!(ts[0].pos.char_len, 5); // "hello"

    assert_eq!(ts[1].pos.char_off, 7);
    assert_eq!(ts[1].pos.char_len, 4); // "this"

    assert_eq!(ts[2].pos.char_off, 12);
    assert_eq!(ts[2].pos.char_len, 2); // "is"

    assert_eq!(ts[3].pos.char_off, 15);
    assert_eq!(ts[3].pos.char_len, 4); // "test"
}

#[test]
fn test_token_word_offsets() {
    let ts = tokens("tests/fixtures/dir1/s1.txt");
    for (i, t) in ts.iter().enumerate() {
        assert_eq!(t.pos.word_off, i, "wrong word offset for token '{}'", t.text);
    }
}

#[test]
fn test_empty_file() {
    let ts = tokens("tests/fixtures/edge_cases/empty.txt");
    assert!(ts.is_empty(), "expected no tokens from empty file");
}

#[test]
fn test_whitespace_only_file() {
    let ts = tokens("tests/fixtures/edge_cases/whitespace_only.txt");
    assert!(
        ts.is_empty(),
        "expected no tokens from whitespace-only file"
    );
}

#[test]
fn test_multiline_byte_offsets() {
    // "hello world\nfoo bar\n"
    // Line 1 contributes 12 bytes ("hello world" = 11 + '\n' = 12)
    // "hello": char.off=0, "world": char.off=6
    // "foo": char.off=12, "bar": char.off=16
    let ts = tokens("tests/fixtures/edge_cases/multiline.txt");
    let texts: Vec<&str> = ts.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(texts, vec!["hello", "world", "foo", "bar"]);

    assert_eq!(ts[0].pos.char_off, 0);
    assert_eq!(ts[0].pos.char_len, 5);

    assert_eq!(ts[1].pos.char_off, 6);
    assert_eq!(ts[1].pos.char_len, 5);

    assert_eq!(ts[2].pos.char_off, 12);
    assert_eq!(ts[2].pos.char_len, 3);

    assert_eq!(ts[3].pos.char_off, 16);
    assert_eq!(ts[3].pos.char_len, 3);
}

#[test]
fn test_multiline_word_offsets() {
    let ts = tokens("tests/fixtures/edge_cases/multiline.txt");
    for (i, t) in ts.iter().enumerate() {
        assert_eq!(t.pos.word_off, i, "wrong word offset for token '{}'", t.text);
    }
}

#[test]
fn test_walkdir_nonexistent_path() {
    let tkn = Tokenizer::new();
    let result = tkn.walkdir(Path::new("tests/fixtures/does_not_exist"));
    assert!(result.is_err(), "expected error for nonexistent directory");
}

#[test]
fn test_walkdir_on_file_path() {
    // read_dir on a file should return an error
    let tkn = Tokenizer::new();
    let result = tkn.walkdir(Path::new("tests/fixtures/sample.rs"));
    assert!(result.is_err(), "expected error when walking a file path");
}

#[test]
fn test_walkdir_empty_directory() {
    let dir = std::env::temp_dir().join("fext_test_empty_dir");
    std::fs::create_dir_all(&dir).unwrap();
    let tkn = Tokenizer::new();
    let mut wd = tkn.walkdir(&dir).unwrap();
    assert!(
        wd.next().is_none(),
        "expected no items from empty directory"
    );
    std::fs::remove_dir(&dir).unwrap();
}

#[test]
fn test_file_not_found_yields_error() {
    let mut ft = FileTokens::new(PathBuf::from("tests/fixtures/nonexistent_file.txt"));
    let result = ft.next();
    assert!(result.is_some(), "expected Some(Err) for missing file");
    assert!(result.unwrap().is_err());
}
