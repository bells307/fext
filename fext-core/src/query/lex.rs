use std::iter::FusedIterator;

#[cfg(test)]
mod tests;

/// Лексический анализатор - принимает на вход строку и выдает токены при
/// использовании как итератора.
pub(crate) struct Lexer<'a> {
    /// Входные данные
    input: &'a str,
    /// Отступ от начала строки - сколько символов уже было обработано
    offset: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self { input, offset: 0 }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Необработанная часть входных данных
            let rest = &self.input[self.offset..];

            match rest.chars().next()? {
                // Пробуем найти ближайшую кавычку
                '"' => match rest[1..].find('"') {
                    Some(i) => {
                        let new_offset = self.offset + i + 2;
                        let s = &self.input[self.offset + 1..new_offset - 1];
                        self.offset = new_offset;
                        break Some(Ok(Token::Phrase(s)));
                    }
                    None => {
                        self.offset = self.input.len();
                        break Some(Err(LexError::UnclosedQuot));
                    }
                },
                '(' => {
                    self.offset += 1;
                    break Some(Ok(Token::LParen));
                }
                ')' => {
                    self.offset += 1;
                    break Some(Ok(Token::RParen));
                }
                c if c.is_whitespace() => {
                    self.offset += c.len_utf8();
                }
                // Перематываем до ближайшего служебного символа или разделителя
                _ => match rest.find(|c: char| {
                    c.is_whitespace() || matches!(c, '"') || matches!(c, '(') || matches!(c, ')')
                }) {
                    Some(i) => {
                        self.offset += i;
                        break Some(Ok(Token::from_str(&rest[..i])));
                    }
                    None => {
                        self.offset = self.input.len();
                        break Some(Ok(Token::from_str(&rest[..])));
                    }
                },
            }
        }
    }
}

impl<'a> FusedIterator for Lexer<'a> {}

/// Токены, которыми оперирует лексический анализатор
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Token<'a> {
    Word(&'a str),
    Phrase(&'a str),
    And,
    Or,
    Not,
    LParen,
    RParen,
}

impl<'a> Token<'a> {
    fn from_str(w: &'a str) -> Self {
        match w {
            "AND" => Token::And,
            "OR" => Token::Or,
            "NOT" => Token::Not,
            _ => Token::Word(w),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum LexError {
    UnclosedQuot,
}

#[derive(Default, Debug)]
enum State {
    #[default]
    Init,
    CollectingWord(String),
    CollectingPhrase(String),
}
