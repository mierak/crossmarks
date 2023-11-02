use anyhow::bail;
use anyhow::Result;
use std::fs;

use clap::{Args, Parser};
use nom::branch::alt;
use nom::bytes::complete::{take_till1, take_while1};
use nom::character::complete::space1;
use nom::sequence::Tuple;
use nom::IResult;

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
        for bookmark in &$bookmarks {
            result.push_str(&format!(concat!($fmt, "\n"), bookmark.alias, bookmark.path));
        }
        fs::write($output_path, result)
    }};
}

fn main() -> Result<()> {
    let args = Config::parse();
    let content = fs::read_to_string(args.bookmarks_file)?;
    let bookmarks = match content
        .lines()
        .filter_map(|line| {
            if line.trim_start().starts_with('#') {
                None
            } else {
                Some(bookmark(line))
            }
        })
        .map(|v| v.map(|v| v.1))
        .collect::<Result<Vec<Bookmark<'_>>, nom::Err<nom::error::VerboseError<&str>>>>()
    {
        Ok(v) => v,
        Err(err) => bail!(err.to_string()),
    };

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

#[derive(Debug, PartialEq)]
struct Bookmark<'a> {
    alias: &'a str,
    path: &'a str,
}

fn till_space(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    take_while1(|c| c != ' ')(input)
}

fn till_whitespace_or_hash(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    take_till1(|c: char| c.is_whitespace() || c == '#')(input)
}

fn quote(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    take_while1(|c| c == '"')(input)
}

fn with_simple_path(input: &str) -> IResult<&str, Bookmark<'_>, nom::error::VerboseError<&str>> {
    let (rest, (alias, _, path)) = (till_space, space1, till_whitespace_or_hash).parse(input)?;
    Ok((rest, Bookmark { alias, path }))
}

fn with_quoted_path(input: &str) -> IResult<&str, Bookmark<'_>, nom::error::VerboseError<&str>> {
    let (rest, (alias, _, _, path, _)) =
        (till_space, space1, quote, take_till1(|c| c == '"'), quote).parse(input)?;
    Ok((rest, Bookmark { alias, path }))
}

fn bookmark(input: &str) -> IResult<&str, Bookmark<'_>, nom::error::VerboseError<&str>> {
    alt((with_quoted_path, with_simple_path))(input)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::bookmark;

    #[test]
    fn only_considers_till_second_space() {
        let input = "a b c";

        let result = bookmark(input).unwrap().1;

        assert_eq!(result.alias, "a");
        assert_eq!(result.path, "b");
    }

    #[test]
    fn ignores_comment() {
        let input = "a b# c";

        let result = bookmark(input).unwrap().1;

        assert_eq!(result.alias, "a");
        assert_eq!(result.path, "b");
    }

    #[test]
    fn correctly_handles_quoted_path() {
        let input = r#"a "test test #test" asdf"#;

        let result = bookmark(input).unwrap().1;

        assert_eq!(result.alias, "a");
        assert_eq!(result.path, "test test #test");
    }
}
