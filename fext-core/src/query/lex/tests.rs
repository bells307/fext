use super::{LexError, Lexer, Token};

fn lex(input: &str) -> Result<Vec<Token<'_>>, LexError> {
    Lexer::new(input).collect()
}

#[test]
fn words() {
    assert_eq!(
        lex("a b"),
        Ok(vec![Token::Word("a".into()), Token::Word("b".into())])
    );
}

#[test]
fn word_and_phrase() {
    assert_eq!(
        lex(r#"a "b c""#),
        Ok(vec![Token::Word("a".into()), Token::Phrase("b c".into())])
    );
}

#[test]
fn parens() {
    assert_eq!(
        lex("(a)"),
        Ok(vec![Token::LParen, Token::Word("a".into()), Token::RParen])
    );
}

#[test]
fn and_keyword_case_sensitive() {
    assert_eq!(
        lex("and AND"),
        Ok(vec![Token::Word("and".into()), Token::And])
    );
}

#[test]
fn unclosed_quote() {
    assert_eq!(lex(r#""foo"#), Err(LexError::UnclosedQuot));
}

#[test]
fn newline_separates_words() {
    assert_eq!(
        lex("a\nb"),
        Ok(vec![Token::Word("a".into()), Token::Word("b".into())])
    );
}

#[test]
fn crlf_around_keyword() {
    assert_eq!(
        lex("a\r\nAND b"),
        Ok(vec![
            Token::Word("a".into()),
            Token::And,
            Token::Word("b".into()),
        ])
    );
}

#[test]
fn newline_inside_phrase_is_literal() {
    assert_eq!(lex("\"a\nb\""), Ok(vec![Token::Phrase("a\nb".into())]));
}

#[test]
fn phrase_right_after_word() {
    assert_eq!(
        lex(r#"ab"cd ef""#),
        Ok(vec![
            Token::Word("ab".into()),
            Token::Phrase("cd ef".into()),
        ])
    );
}

#[test]
fn word_after_phrase() {
    assert_eq!(
        lex(r#""ab cd" ef"#),
        Ok(vec![
            Token::Phrase("ab cd".into()),
            Token::Word("ef".into()),
        ])
    );
}

#[test]
fn empty_input() {
    assert_eq!(lex(""), Ok(vec![]));
}

#[test]
fn spaces_only() {
    assert_eq!(lex("   "), Ok(vec![]));
}

#[test]
fn multiple_spaces_between_words() {
    assert_eq!(
        lex("a   b"),
        Ok(vec![Token::Word("a".into()), Token::Word("b".into())])
    );
}

#[test]
fn and_or_operators() {
    assert_eq!(
        lex("a AND b OR c"),
        Ok(vec![
            Token::Word("a".into()),
            Token::And,
            Token::Word("b".into()),
            Token::Or,
            Token::Word("c".into()),
        ])
    );
}

#[test]
fn not_operator() {
    assert_eq!(lex("NOT a"), Ok(vec![Token::Not, Token::Word("a".into())]));
}

#[test]
fn not_alone_at_end_of_input() {
    assert_eq!(lex("NOT"), Ok(vec![Token::Not]));
}

#[test]
fn and_not_combination() {
    assert_eq!(
        lex("a AND NOT b"),
        Ok(vec![
            Token::Word("a".into()),
            Token::And,
            Token::Not,
            Token::Word("b".into()),
        ])
    );
}

#[test]
fn double_not() {
    assert_eq!(
        lex("NOT NOT a"),
        Ok(vec![Token::Not, Token::Not, Token::Word("a".into())])
    );
}

#[test]
fn not_before_paren() {
    assert_eq!(
        lex("NOT(a OR b)"),
        Ok(vec![
            Token::Not,
            Token::LParen,
            Token::Word("a".into()),
            Token::Or,
            Token::Word("b".into()),
            Token::RParen,
        ])
    );
}

#[test]
fn not_before_phrase() {
    assert_eq!(
        lex(r#"NOT"a b""#),
        Ok(vec![Token::Not, Token::Phrase("a b".into())])
    );
}

#[test]
fn word_with_not_prefix() {
    assert_eq!(lex("NOTHING"), Ok(vec![Token::Word("NOTHING".into())]));
}

#[test]
fn not_inside_phrase_is_literal() {
    assert_eq!(lex(r#""NOT a""#), Ok(vec![Token::Phrase("NOT a".into())]));
}

#[test]
fn mixed_case_not_is_word() {
    assert_eq!(
        lex("Not a"),
        Ok(vec![Token::Word("Not".into()), Token::Word("a".into())])
    );
}

#[test]
fn keyword_at_end_of_input() {
    assert_eq!(lex("a AND"), Ok(vec![Token::Word("a".into()), Token::And]));
}

#[test]
fn keyword_terminated_by_paren() {
    assert_eq!(
        lex("AND(a)"),
        Ok(vec![
            Token::And,
            Token::LParen,
            Token::Word("a".into()),
            Token::RParen,
        ])
    );
}

#[test]
fn word_containing_keyword_prefix() {
    assert_eq!(lex("ANDROID"), Ok(vec![Token::Word("ANDROID".into())]));
}

#[test]
fn lowercase_keywords_are_words() {
    assert_eq!(
        lex("and or not"),
        Ok(vec![
            Token::Word("and".into()),
            Token::Word("or".into()),
            Token::Word("not".into()),
        ])
    );
}

#[test]
fn nested_parens() {
    assert_eq!(
        lex("((a))"),
        Ok(vec![
            Token::LParen,
            Token::LParen,
            Token::Word("a".into()),
            Token::RParen,
            Token::RParen,
        ])
    );
}

#[test]
fn empty_parens() {
    assert_eq!(lex("()"), Ok(vec![Token::LParen, Token::RParen]));
}

#[test]
fn empty_phrase() {
    assert_eq!(lex(r#""""#), Ok(vec![Token::Phrase("".into())]));
}

#[test]
fn keywords_inside_phrase_are_literal() {
    assert_eq!(
        lex(r#""a AND b""#),
        Ok(vec![Token::Phrase("a AND b".into())])
    );
}

#[test]
fn parens_inside_phrase_are_literal() {
    assert_eq!(lex(r#""(a)""#), Ok(vec![Token::Phrase("(a)".into())]));
}

#[test]
fn adjacent_phrases() {
    assert_eq!(
        lex(r#""a""b""#),
        Ok(vec![Token::Phrase("a".into()), Token::Phrase("b".into())])
    );
}

#[test]
fn unicode_words() {
    assert_eq!(
        lex("поиск файл"),
        Ok(vec![
            Token::Word("поиск".into()),
            Token::Word("файл".into()),
        ])
    );
}

#[test]
fn word_with_punctuation() {
    assert_eq!(lex("foo-bar"), Ok(vec![Token::Word("foo-bar".into())]));
}

#[test]
fn tab_separates_words() {
    assert_eq!(
        lex("a\tb"),
        Ok(vec![Token::Word("a".into()), Token::Word("b".into())])
    );
}

#[test]
fn unclosed_quote_empty() {
    assert_eq!(lex(r#"""#), Err(LexError::UnclosedQuot));
}

#[test]
fn unclosed_quote_after_word() {
    assert_eq!(lex(r#"a "b c"#), Err(LexError::UnclosedQuot));
}

#[test]
fn complex_query() {
    assert_eq!(
        lex(r#"(foo OR "bar baz") AND qux"#),
        Ok(vec![
            Token::LParen,
            Token::Word("foo".into()),
            Token::Or,
            Token::Phrase("bar baz".into()),
            Token::RParen,
            Token::And,
            Token::Word("qux".into()),
        ])
    );
}
