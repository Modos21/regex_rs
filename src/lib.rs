pub(crate) mod lexer;
pub(crate) mod parser;
pub(crate) mod token;
pub mod regex;

#[cfg(test)]
mod tests {
    use crate::token::Token;

    #[test]
    fn it_works() {
        let tok1 = Token::Any;
        let tok2 = Token::Char('a');

        assert_ne!(tok1, tok2);
    }
}
