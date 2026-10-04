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
    fn new(text: &'a str) -> Self {
        Self {
            tokens: Lexer::new(text).peekable(),
        }
    }

    fn parse(&mut self) -> Result<Option<QueryAst<'a>>, ParseError> {
        let mut acc = Accumulator::new();

        loop {
            match self.tokens.peek() {
                Some(res) => match res {
                    Ok(Token::Or) => {
                        self.tokens.next();
                    }
                    Ok(Token::LParen) => todo!(),
                    Ok(Token::RParen) => todo!(),
                    Ok(Token::Not) => todo!(),
                    Err(..) => {
                        let Some(Err(err)) = self.tokens.next() else {
                            unreachable!()
                        };
                        return Err(ParseError::Lex(err));
                    }
                    _ => {
                        if let Some(and) = self.parse_and()? {
                            acc.push(and);
                        }
                    }
                },
                None => break,
            }
        }

        match acc {
            Accumulator::Empty => Ok(None),
            Accumulator::One(one) => Ok(Some(one)),
            Accumulator::Many(vec) => Ok(Some(QueryAst::Or(vec))),
        }
    }

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
    fn parse_and(&mut self) -> Result<Option<QueryAst<'a>>, ParseError> {
        let mut acc = Accumulator::new();

        loop {
            match self.tokens.peek() {
                Some(res) => match res {
                    Ok(Token::And) => {
                        self.tokens.next();
                    }
                    Ok(Token::Or) => break,
                    Err(..) => {
                        let Some(Err(err)) = self.tokens.next() else {
                            unreachable!()
                        };
                        return Err(ParseError::Lex(err));
                    }
                    _ => {
                        let q = self.parse_factor()?;
                        acc.push(q);
                    }
                },
                _ => break,
            }
        }

        match acc {
            Accumulator::Empty => Ok(None),
            Accumulator::One(one) => Ok(Some(one)),
            Accumulator::Many(vec) => Ok(Some(QueryAst::And(vec))),
        }
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

#[derive(Debug, PartialEq)]
enum ParseError {
    Lex(LexError),
    UnexpectedEnd,
    UnexpectedToken,
}

enum Accumulator<T> {
    Empty,
    One(T),
    Many(Vec<T>),
}

impl<T> Accumulator<T> {
    fn new() -> Self {
        Self::Empty
    }

    fn push(&mut self, v: T) {
        match self {
            Self::Empty => *self = Self::One(v),
            Self::Many(items) => items.push(v),
            Self::One(_) => {
                let items = Vec::with_capacity(2);
                let old = std::mem::replace(self, Self::Many(items));

                if let (Self::One(first), Self::Many(items)) = (old, self) {
                    items.push(first);
                    items.push(v);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(s: &str) -> QueryAst<'_> {
        QueryAst::Word(s)
    }

    fn and(v: Vec<QueryAst>) -> QueryAst {
        QueryAst::And(v)
    }

    fn or(v: Vec<QueryAst>) -> QueryAst {
        QueryAst::Or(v)
    }

    #[test]
    fn parse_test() {
        let cases = [
            (
                "hello world",
                Ok(Some(and(vec![word("hello"), word("world")]))),
            ),
            (
                "hello AND world",
                Ok(Some(and(vec![word("hello"), word("world")]))),
            ),
            (
                "hello OR world",
                Ok(Some(or(vec![word("hello"), word("world")]))),
            ),
            (
                "hello new OR world",
                Ok(Some(or(vec![
                    and(vec![word("hello"), word("new")]),
                    word("world"),
                ]))),
            ),
            (
                "hello AND brave new OR world",
                Ok(Some(or(vec![
                    and(vec![word("hello"), word("brave"), word("new")]),
                    word("world"),
                ]))),
            ),
            ("", Ok(None)),
        ];

        for (text, exp) in cases {
            let res = QueryParser::new(text).parse();
            if res != exp {
                panic!("fail: {}\nexpected: {:?}\ngot: {:?}", text, exp, res);
            }
        }
    }
}
