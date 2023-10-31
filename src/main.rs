use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::{
    fs,
    io::{BufRead, BufReader},
};

use clap::{Args, Parser};

#[derive(Parser)]
struct Config {
    #[arg(short = 'i', long = "input")]
    bookmarks_file: String,
    #[command(flatten)]
    outputs: Outputs,
}

#[derive(Args, Clone)]
#[group(required = true, multiple = true)]
struct Outputs {
    #[arg(short = 'l', long = "lf")]
    lf_file: Option<String>,
    #[arg(short = 'z', long = "zsh")]
    zsh_named_dirs_file: Option<String>,
    #[arg(short = 'c', long = "cd-alias")]
    cd_aliases_file: Option<String>,
}

macro_rules! write_formatted {
    ($bookmarks:ident, $output_path:ident, $fmt:literal) => {{
        let mut result = String::new();
        for bookmark in &$bookmarks.0 {
            result.push_str(&format!(
                concat!($fmt, "\n"),
                bookmark.alias(),
                bookmark.path()
            ));
        }
        fs::write($output_path, result)
    }};
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Config::parse();
    let input = fs::File::open(args.bookmarks_file)?;
    let bookmarks: Bookmarks = input.try_into()?;

    if let Some(output_path) = args.outputs.lf_file {
        write_formatted!(bookmarks, output_path, "map g{} cd {}")?;
    }
    if let Some(output_path) = args.outputs.zsh_named_dirs_file {
        write_formatted!(bookmarks, output_path, "hash -d {}={}")?;
    }
    if let Some(output_path) = args.outputs.cd_aliases_file {
        write_formatted!(bookmarks, output_path, r#"alias cd{}="{}""#)?;
    }

    Ok(())
}

struct Bookmark {
    _input: String,
    first_space_idx: usize,
    second_space_idx: usize,
}

impl std::fmt::Debug for Bookmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ _input: {}, alias: \"{}\", path: \"{}\", first_space_idx: {}, second_space_idx: {} }}",
            self._input,
            self.alias(),
            self.path(),
            self.first_space_idx,
            self.second_space_idx,
        )
    }
}

#[derive(Debug)]
enum BookmarkError {
    CommentLine(String),
    InvalidFormat(String),
}

impl Display for BookmarkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookmarkError::CommentLine(line) => write!(f, "Unexpected comment line: '{line}'"),
            BookmarkError::InvalidFormat(line) => write!(f, "Invalid format: '{line}'"),
        }
    }
}

impl std::error::Error for BookmarkError {}

impl TryFrom<String> for Bookmark {
    type Error = BookmarkError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim_start().starts_with('#') {
            return Err(BookmarkError::CommentLine(value));
        }

        if value.split(' ').count() < 2 {
            return Err(BookmarkError::InvalidFormat(value));
        }

        let first_space_idx = value.find(' ').unwrap();
        let second_space_idx = value[first_space_idx + 1..]
            .find(' ')
            .unwrap_or(value.len());

        if value[..first_space_idx].is_empty()
            || value[first_space_idx + 1..second_space_idx].is_empty()
        {
            return Err(BookmarkError::InvalidFormat(value));
        }

        Ok(Self {
            _input: value,
            first_space_idx,
            second_space_idx,
        })
    }
}

impl Bookmark {
    pub fn path(&self) -> &str {
        &self._input[self.first_space_idx + 1..self.second_space_idx]
    }
    pub fn alias(&self) -> &str {
        &self._input[..self.first_space_idx]
    }
}

struct Bookmarks(Vec<Bookmark>);
impl TryFrom<File> for Bookmarks {
    type Error = BookmarkError;

    fn try_from(input: File) -> std::result::Result<Self, Self::Error> {
        let read = BufReader::new(input);
        let bms = read
            .lines()
            .filter_map(|v| -> Option<Result<Bookmark, BookmarkError>> {
                v.map_or(None, |v| match v.try_into() {
                    Ok(v) => Some(Ok(v)),
                    Err(e) => match e {
                        BookmarkError::CommentLine(_) => None,
                        BookmarkError::InvalidFormat(line) => {
                            Some(Err(BookmarkError::InvalidFormat(line)))
                        }
                    },
                })
            })
            .collect::<Result<Vec<Bookmark>, BookmarkError>>()?;
        Ok(Bookmarks(bms))
    }
}
