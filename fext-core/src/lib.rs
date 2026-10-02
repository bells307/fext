pub use index::InvertedIndex;

mod index;
mod query;

use std::hash::Hash;

/// Позиция вхождения токена в документе
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Position {
    pub char_off: usize,
    pub char_len: usize,
    pub word_off: usize,
}
