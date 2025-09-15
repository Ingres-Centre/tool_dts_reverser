use logos::{Logos, Span};
use std::num::{IntErrorKind, ParseIntError};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalErrorKind {
    #[default]
    InvalidToken,
    InvalidInteger(ParseIntError),
}

impl From<ParseIntError> for LexicalErrorKind {
    fn from(value: ParseIntError) -> Self {
        Self::InvalidInteger(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexicalError<'a> {
    kind: LexicalErrorKind,
    input: &'a str,
    span: Span,
}

impl<'a> std::fmt::Display for LexicalError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let line = self.input[..self.span.start]
            .chars()
            .filter(|&ch| ch == '\n')
            .count()
            + 1;

        let column = self.span.start - self.input[..self.span.start].rfind("\n").unwrap_or(0);

        let position = format!("line {}, column {}", line, column);

        match &self.kind {
            LexicalErrorKind::InvalidToken => write!(
                f,
                "Invalid token \"{}\" at {}",
                &self.input[self.span.start..self.span.end],
                position
            )?,
            LexicalErrorKind::InvalidInteger(inner) => write!(
                f,
                "Invalid number {} at {}: {}",
                &self.input[self.span.start..self.span.end],
                position,
                match inner.kind() {
                    IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => "overflow",
                    _ => "unknown",
                }
            )?,
        };

        Ok(())
    }
}

fn string_from_lexer<'a>(lex: &mut logos::Lexer<'a, Token<'a>>) -> &'a str {
    let slice = lex.slice();
    &slice[1..slice.len() - 1]
}

#[derive(Clone, Debug, PartialEq, Logos)]
#[logos(error = LexicalErrorKind)]
#[logos(skip r"[\s\t\n\f]+")]
pub enum Token<'a> {
    // #[regex(r"//.*")]
    // SingleLineComment(&'a str),
    //
    // #[regex(r"\/\*[^*]*\*+(?:[^\/*][^*]*\*+)*\/")]
    // MultiLineComment(&'a str),
    #[token("=")]
    Eq,

    #[token(":")]
    Colon,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[token(".")]
    Period,

    #[token("(")]
    OpenPth,

    #[token(")")]
    ClosePth,

    #[token("[")]
    OpenBracket,

    #[token("]")]
    CloseBracket,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("<")]
    OpenAngle,

    #[token(">")]
    CloseAngle,

    #[token("true", |_| true)]
    #[token("false", |_| false)]
    Boolean(bool),

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse())]
    #[regex(r"0x[0-9a-fA-F]{1,16}", |lex| i64::from_str_radix(&lex.slice()[2..], 16))]
    Integer(i64),

    #[regex(r"[0-9a-fA-F]{2}", |lex| u8::from_str_radix(lex.slice(), 16))]
    Byte(u8),

    #[regex(r#"'((?:[^'\n]|(?:\\\'))*)'"#, string_from_lexer)]
    #[regex(r#""((?:[^"\n]|(?:\\\"))*)""#, string_from_lexer)]
    String(&'a str),

    #[regex(r"[#a-zA-Z_/][@,/\-a-zA-Z_0-9]*", priority = 0)]
    Ident(&'a str),

    #[regex(r"/dts-v([0-9])/", |lex| lex.slice()[6..7].parse())]
    Header(u32),
}

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Lexer<'input> {
    inner: logos::SpannedIter<'input, Token<'input>>,
}

impl<'input> Lexer<'input> {
    pub fn new(src: &'input str) -> Self {
        Self {
            inner: Token::lexer(src).spanned(),
        }
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token<'input>, usize, LexicalError<'input>>;

    fn next(&mut self) -> Option<Self::Item> {
        let (tok, span) = self.inner.next()?;

        Some(
            tok.map(|tok| (span.start, tok, span.end))
                .map_err(|kind| LexicalError {
                    kind,
                    input: self.inner.source(),
                    span,
                }),
        )
    }
}
