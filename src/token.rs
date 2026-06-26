use std::range::RangeInclusive;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    /// a single specific character
    Char(char),

    /// a range of characters separated by a -
    /// most commonly used for a-z, A-Z, or 0-9
    Range(RangeInclusive<char>),

    /// the token - between two chars
    RangeIndicator,

    /// the token .
    Any,

    /// any kind of whitespace
    /// includes \s \S and any whitespace char
    Whitespace(Whitespace),

    /// the token \d
    Digit,

    /// the token \D
    NonDigit,

    /// the token \w
    WordChar,

    /// the token \W
    NonWordChar,

    /// any repetition token
    /// candidates are: * + ? {n}
    Repeat(Repetition),

    /// the token |
    Or,

    /// the token [
    InclSet,

    /// the token [^
    ExclSet,

    /// the token ]
    SetEnd,

    /// the token (
    GroupStart,

    /// the token )
    GroupEnd,

    /// the token \n
    Newline,

    /// the token \t
    Tab,

    /// the token \0
    NullChar,

    /// start of a line represented by the token ^
    SOL,

    /// end of a line represented by the token $
    EOL,

    /// end of file
    EOF
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub(crate) enum Whitespace {
    AnyWS,
    NonWS,
    Repeated(u32),
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub(crate) enum Repetition {
    /// the token *
    ZeroOrMore,

    /// the token +
    OneOrMore,

    /// the token ?
    ZeroOrOne,

    /// repetition for an exact number of times
    /// written as {n}
    ExactlyN(u32),
}
