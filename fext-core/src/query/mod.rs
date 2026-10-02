//! Язык запросов:
//! query := or_expr (* запрос целиком - это одно OR-выражение *)
//! or_expr := and_expr ( "OR" and_expr )* (* OR-выражение - это одно или несколько AND-выражений,
//! соединенных словом OR *)
//! and_expr := factor ( "AND"? factor )* (* AND-выражение - это один или несколько факторов,
//! соединенных словом AND, причем само слово AND можно опускать *)
//! factor := WORD | PHRASE | "NOT" factor | "(" or_expr ")" (* Фактор - это одно из четырех: слово,
//! фраза, отрицание фактора или выражение в скобках *)

mod lex;

use crate::query::lex::{LexError, Lexer, Token};
use std::iter::Peekable;

struct QueryParser<'a> {
    tokens: Peekable<Lexer<'a>>,
}

impl<'a> QueryParser<'a> {
    fn parse_factor(&mut self) -> Result<QueryAst<'a>, ParseError> {
        match self.tokens.next().transpose().map_err(ParseError::Lex)? {
            Some(Token::Word(w)) => Ok(QueryAst::Word(w)),
            Some(Token::Phrase(p)) => Ok(QueryAst::Phrase(p)),
            Some(Token::Not) => self.parse_factor().map(Box::new).map(QueryAst::Not),
            Some(_) => Err(ParseError::UnexpectedToken),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    /// Парсинг последовательности
    fn parse_and(&mut self) -> Result<QueryAst<'a>, ParseError> {
        // hello world
        let mut and = vec![self.parse_factor()?];

        loop {
            if let Some(Ok(token)) = self.tokens.peek() {
                match token {
                    Token::And => {}
                    Token::Or => break,
                    _ => {
                        let q = self.parse_factor()?;
                        and.push(q);
                    }
                }
            }
        }

        Ok(QueryAst::And(and))
    }
}

#[derive(Debug, Eq, PartialEq)]
enum QueryAst<'a> {
    Word(&'a str),
    Phrase(&'a str),
    And(Vec<QueryAst<'a>>),
    Or(Vec<QueryAst<'a>>),
    Not(Box<QueryAst<'a>>),
}

#[derive(Debug)]
enum ParseError {
    Lex(LexError),
    UnexpectedEnd,
    UnexpectedToken,
}

fn parse(text: &str) -> Result<QueryAst, ParseError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        let text = "hello world";
        let q_ast = parse(text).unwrap();
        let exp = QueryAst::And(vec![
            QueryAst::Word("hello".into()),
            QueryAst::Word("world".into()),
        ]);
        assert_eq!(q_ast, exp);
    }
}
