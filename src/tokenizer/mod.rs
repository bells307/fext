#[cfg(test)]
mod tests;

use fext_core::Position;
use std::{
    collections::VecDeque,
    fs::{self, File, ReadDir},
    io::{self, BufRead, BufReader, Lines},
    iter::FusedIterator,
    path::{Path, PathBuf},
};

pub struct Tokenizer {}

impl Tokenizer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn walkdir(&self, path: &Path) -> io::Result<WalkDir> {
        WalkDir::try_new(path)
    }
}

/// Итератор для обхода директорий
pub struct WalkDir {
    stack: Vec<ReadDir>,
}

impl WalkDir {
    fn try_new(path: &Path) -> io::Result<Self> {
        let rd = fs::read_dir(path)?;
        Ok(Self { stack: vec![rd] })
    }
}

impl Iterator for WalkDir {
    type Item = io::Result<FileTokens>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Берем из стека верхнюю директорию. Если стек пустой, значит новых файлов/директорий больше не будет
            let mut top = self.stack.pop()?;

            match top.next() {
                Some(Ok(ent)) => match ent.file_type() {
                    Ok(ft) => {
                        if ft.is_file() {
                            self.stack.push(top);
                            return Some(Ok(FileTokens::new(ent.path())));
                        } else if ft.is_dir() {
                            match fs::read_dir(ent.path()) {
                                Ok(rd) => {
                                    // Идем в глубину
                                    self.stack.push(top);
                                    self.stack.push(rd);
                                    continue;
                                }
                                Err(e) => {
                                    self.stack = Vec::new();
                                    return Some(Err(e));
                                }
                            }
                        } else {
                            self.stack.push(top);
                        }
                    }
                    Err(e) => {
                        self.stack = Vec::new();
                        return Some(Err(e));
                    }
                },
                Some(Err(e)) => {
                    self.stack = Vec::new();
                    return Some(Err(e));
                }
                // Мы уже прочитали всю директорию - надо переходить к следующей
                None => {}
            }
        }
    }
}

impl FusedIterator for WalkDir {}

/// Итератор по токенам файла
pub struct FileTokens {
    /// Путь к файлу
    path: PathBuf,
    /// Буфер токенов
    buf: VecDeque<Token>,
    byte_off: usize,
    word_off: usize,
    lines: Option<Lines<BufReader<File>>>,
}

impl FileTokens {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            buf: VecDeque::new(),
            byte_off: 0,
            word_off: 0,
            lines: None,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn read_next_token(&mut self) -> io::Result<Option<Token>> {
        if let Some(t) = self.buf.pop_front() {
            return Ok(Some(t));
        }

        let lines = match &mut self.lines {
            Some(ln) => ln,
            None => {
                self.lines = Some(BufReader::new(File::open(&self.path)?).lines());
                self.lines.as_mut().unwrap()
            }
        };

        loop {
            match lines.next() {
                Some(Ok(ln)) => {
                    let mut in_word = None;

                    for (i, c) in ln.char_indices() {
                        if is_word_breaker(c) {
                            if let Some(word_start) = in_word {
                                push_word(
                                    &mut self.buf,
                                    self.byte_off,
                                    self.word_off,
                                    &ln[word_start..i],
                                    word_start,
                                );
                                self.word_off += 1;
                                in_word = None;
                            }
                        } else if in_word.is_none() {
                            in_word = Some(i);
                        }
                    }

                    if let Some(word_start) = in_word {
                        push_word(
                            &mut self.buf,
                            self.byte_off,
                            self.word_off,
                            &ln[word_start..],
                            word_start,
                        );
                        self.word_off += 1;
                    }
                    // +1 за `\n`
                    self.byte_off += ln.len() + 1;
                    if let Some(t) = self.buf.pop_front() {
                        return Ok(Some(t));
                    }
                }
                Some(Err(e)) => return Err(e),
                None => return Ok(None),
            }
        }
    }
}

fn is_word_breaker(c: char) -> bool {
    c.is_whitespace() || c.is_ascii_punctuation()
}

fn push_word(
    buf: &mut VecDeque<Token>,
    byte_off: usize,
    word_off: usize,
    word: &str,
    byte_off_in_line: usize,
) {
    let text = word.to_lowercase();
    let len = text.len();

    buf.push_back(Token {
        text,
        pos: Position {
            char_off: byte_off + byte_off_in_line,
            char_len: len,
            word_off,
        },
    });
}

impl Iterator for FileTokens {
    type Item = io::Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_next_token() {
            Ok(Some(t)) => Some(Ok(t)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[derive(Debug)]
pub struct Token {
    pub text: String,
    pub pos: Position,
}
