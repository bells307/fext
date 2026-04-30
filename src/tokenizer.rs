use std::{
    collections::VecDeque,
    fs::{self, File, ReadDir},
    io::{self, BufRead, BufReader, Lines},
    iter::FusedIterator,
    marker::PhantomData,
    path::{Path, PathBuf},
};

struct Tokenizer {}

impl Tokenizer {
    fn new() -> Self {
        Self {}
    }

    /// Обход директории
    fn walkdir<'a>(&'a self, path: &'a Path) -> io::Result<WalkDir<'a>> {
        WalkDir::try_new(path)
    }
}

/// Итератор для обхода директорий
struct WalkDir<'a> {
    stack: Vec<ReadDir>,
    _p: PhantomData<&'a ()>,
}

impl<'a> WalkDir<'a> {
    fn try_new(path: &Path) -> io::Result<Self> {
        let rd = fs::read_dir(path)?;

        Ok(Self {
            stack: vec![rd],
            _p: PhantomData,
        })
    }
}

impl<'a> Iterator for WalkDir<'a> {
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
                                    self.stack.clear();
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

impl<'a> FusedIterator for WalkDir<'a> {}

/// Итератор по токенам файла
struct FileTokens {
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

    fn path(&self) -> &Path {
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
    buf.push_back(Token {
        text: word.to_string(),
        pos: CompositePosition {
            char: Position {
                off: byte_off + byte_off_in_line,
                len: word.len(),
            },
            word: Position {
                off: word_off,
                len: 1,
            },
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
struct Token {
    text: String,
    pos: CompositePosition,
}

#[derive(Clone, Copy, Debug)]
struct Position {
    off: usize,
    len: usize,
}

#[derive(Clone, Copy, Debug)]
struct CompositePosition {
    char: Position,
    word: Position,
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::Tokenizer;
    use std::{collections::HashMap, path::Path};

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
}
