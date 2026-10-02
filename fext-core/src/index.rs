use crate::Position;
use std::{collections::HashMap, hash::Hash};

/// Обратный индекс
pub struct InvertedIndex<D> {
    /// Текст токена -> идентификатор документа -> позиции всех вхождений токена в документе
    map: HashMap<String, HashMap<D, Vec<Position>>>,
}

impl<D> InvertedIndex<D>
where
    D: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Добавить вхождение токена в документ
    pub fn push(&mut self, text: &str, doc_id: D, pos: Position) {
        self.map
            .entry(text.to_string())
            .or_default()
            .entry(doc_id)
            .or_default()
            .push(pos);
    }

    /// Вернуть карту "документ -> позиции" для токена, либо None если токен не встречается
    pub fn search(&self, text: &str) -> Option<&HashMap<D, Vec<Position>>> {
        self.map.get(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(char_off: usize, char_len: usize, word_off: usize) -> Position {
        Position {
            char_off,
            char_len,
            word_off,
        }
    }

    #[test]
    fn push_and_search_single() {
        let mut idx = InvertedIndex::new();
        idx.push("hello", 1, pos(0, 5, 0));

        let result = idx.search("hello").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&1).unwrap(), &vec![pos(0, 5, 0)]);
    }

    #[test]
    fn search_missing_token_returns_none() {
        let mut idx = InvertedIndex::new();
        idx.push("hello", 1, pos(0, 5, 0));

        assert!(idx.search("world").is_none());
    }

    #[test]
    fn same_token_multiple_files() {
        let mut idx = InvertedIndex::new();
        idx.push("foo", 1, pos(0, 3, 0));
        idx.push("foo", 2, pos(10, 3, 2));

        let result = idx.search("foo").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&1).unwrap(), &vec![pos(0, 3, 0)]);
        assert_eq!(result.get(&2).unwrap(), &vec![pos(10, 3, 2)]);
    }

    #[test]
    fn same_token_same_file_collects_positions() {
        let mut idx = InvertedIndex::new();
        idx.push("dup", 1, pos(0, 3, 0));
        idx.push("dup", 1, pos(8, 3, 2));

        let result = idx.search("dup").unwrap();
        assert_eq!(result.get(&1).unwrap(), &vec![pos(0, 3, 0), pos(8, 3, 2)]);
    }

    #[test]
    fn different_tokens_are_independent() {
        let mut idx = InvertedIndex::new();
        idx.push("foo", 1, pos(0, 3, 0));
        idx.push("bar", 2, pos(0, 3, 0));

        let foo = idx.search("foo").unwrap();
        assert!(foo.contains_key(&1));
        assert!(!foo.contains_key(&2));

        let bar = idx.search("bar").unwrap();
        assert!(bar.contains_key(&2));
        assert!(!bar.contains_key(&1));
    }
}
