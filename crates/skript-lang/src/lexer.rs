//! Lexer for the Skript language.
//!
//! Skript is indentation-based (like Python) and uses natural English-like syntax.
//! We use a simple hand-written lexer for better control over indentation handling.

// We track positions by UTF-8 character boundaries, so string slicing is safe here.
#![allow(clippy::string_slice)]

use std::fmt;

/// A span in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<Span> for std::ops::Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

/// A value with its source span.
pub type Spanned<T> = (T, Span);

/// Tokens in the Skript language.
#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    // Literals
    Number(f64),
    String(&'src str),

    // Identifiers and keywords
    Ident(&'src str),

    // Keywords
    On,
    If,
    Else,
    Loop,
    While,
    Set,
    To,
    Send,
    Broadcast,
    Cancel,
    Stop,
    Return,
    True,
    False,
    And,
    Or,
    Not,
    Is,
    The,
    A,
    An,
    Of,
    In,
    At,
    From,
    For,
    With,
    Without,
    Command,
    Trigger,
    Permission,
    Aliases,
    Function,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // Delimiters
    Colon,
    Comma,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // Structure
    Newline,
    Indent,
    Dedent,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Ident(s) => write!(f, "{s}"),
            Self::On => write!(f, "on"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::Loop => write!(f, "loop"),
            Self::While => write!(f, "while"),
            Self::Set => write!(f, "set"),
            Self::To => write!(f, "to"),
            Self::Send => write!(f, "send"),
            Self::Broadcast => write!(f, "broadcast"),
            Self::Cancel => write!(f, "cancel"),
            Self::Stop => write!(f, "stop"),
            Self::Return => write!(f, "return"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
            Self::Not => write!(f, "not"),
            Self::Is => write!(f, "is"),
            Self::The => write!(f, "the"),
            Self::A => write!(f, "a"),
            Self::An => write!(f, "an"),
            Self::Of => write!(f, "of"),
            Self::In => write!(f, "in"),
            Self::At => write!(f, "at"),
            Self::From => write!(f, "from"),
            Self::For => write!(f, "for"),
            Self::With => write!(f, "with"),
            Self::Without => write!(f, "without"),
            Self::Command => write!(f, "command"),
            Self::Trigger => write!(f, "trigger"),
            Self::Permission => write!(f, "permission"),
            Self::Aliases => write!(f, "aliases"),
            Self::Function => write!(f, "function"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Percent => write!(f, "%"),
            Self::Eq => write!(f, "="),
            Self::NotEq => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::Gt => write!(f, ">"),
            Self::LtEq => write!(f, "<="),
            Self::GtEq => write!(f, ">="),
            Self::Colon => write!(f, ":"),
            Self::Comma => write!(f, ","),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),
            Self::Newline => write!(f, "\\n"),
            Self::Indent => write!(f, "INDENT"),
            Self::Dedent => write!(f, "DEDENT"),
        }
    }
}

/// Lexer error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {:?}", self.message, self.span)
    }
}

/// Lex a Skript source string into tokens.
pub fn lex(source: &str) -> Result<Vec<Spanned<Token<'_>>>, LexError> {
    let mut lexer = Lexer::new(source);
    lexer.lex()
}

struct Lexer<'src> {
    source: &'src str,
    pos: usize,
    indent_stack: Vec<usize>,
}

impl<'src> Lexer<'src> {
    fn new(source: &'src str) -> Self {
        Self {
            source,
            pos: 0,
            indent_stack: vec![0],
        }
    }

    fn lex(&mut self) -> Result<Vec<Spanned<Token<'src>>>, LexError> {
        let mut tokens = Vec::new();
        let mut at_line_start = true;

        while self.pos < self.source.len() {
            if at_line_start {
                // Handle indentation at the start of a line
                let indent = self.count_indent();
                let current = *self.indent_stack.last().unwrap_or(&0);

                if indent > current {
                    self.indent_stack.push(indent);
                    tokens.push((Token::Indent, Span::new(self.pos, self.pos)));
                } else {
                    while indent < *self.indent_stack.last().unwrap_or(&0) {
                        self.indent_stack.pop();
                        tokens.push((Token::Dedent, Span::new(self.pos, self.pos)));
                    }
                }
                at_line_start = false;
            }

            // Skip horizontal whitespace (not at line start)
            self.skip_hspace();

            if self.pos >= self.source.len() {
                break;
            }

            let c = self.current();

            // Comment
            if c == '#' {
                self.skip_comment();
                continue;
            }

            // Newline
            if c == '\n' {
                let start = self.pos;
                self.advance();
                tokens.push((Token::Newline, Span::new(start, self.pos)));
                at_line_start = true;
                continue;
            }

            // Token
            let token = self.lex_token()?;
            tokens.push(token);
        }

        // Final dedents
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push((Token::Dedent, Span::new(self.pos, self.pos)));
        }

        Ok(tokens)
    }

    fn current(&self) -> char {
        self.source[self.pos..].chars().next().unwrap_or('\0')
    }

    fn peek(&self) -> char {
        self.source[self.pos..].chars().nth(1).unwrap_or('\0')
    }

    fn advance(&mut self) {
        if self.pos < self.source.len() {
            self.pos += self.current().len_utf8();
        }
    }

    fn count_indent(&mut self) -> usize {
        let mut count = 0;
        while self.pos < self.source.len() {
            match self.current() {
                '\t' => {
                    count += 1; // Treat tab as 1 indent level
                    self.advance();
                }
                ' ' => {
                    count += 1; // Treat space as 1 indent unit
                    self.advance();
                }
                _ => break,
            }
        }
        count
    }

    fn skip_hspace(&mut self) {
        while self.pos < self.source.len() {
            match self.current() {
                ' ' | '\t' => self.advance(),
                _ => break,
            }
        }
    }

    fn skip_comment(&mut self) {
        while self.pos < self.source.len() && self.current() != '\n' {
            self.advance();
        }
    }

    fn lex_token(&mut self) -> Result<Spanned<Token<'src>>, LexError> {
        let start = self.pos;
        let c = self.current();

        // String
        if c == '"' {
            return self.lex_string();
        }

        // Number
        if c.is_ascii_digit() {
            return self.lex_number();
        }

        // Identifier or keyword
        if c.is_alphabetic() || c == '_' {
            return Ok(self.lex_ident());
        }

        // Operators and delimiters
        let token = match c {
            '+' => {
                self.advance();
                Token::Plus
            }
            '-' => {
                self.advance();
                Token::Minus
            }
            '*' => {
                self.advance();
                Token::Star
            }
            '/' => {
                self.advance();
                Token::Slash
            }
            '%' => {
                self.advance();
                Token::Percent
            }
            '!' if self.peek() == '=' => {
                self.advance();
                self.advance();
                Token::NotEq
            }
            '<' if self.peek() == '=' => {
                self.advance();
                self.advance();
                Token::LtEq
            }
            '>' if self.peek() == '=' => {
                self.advance();
                self.advance();
                Token::GtEq
            }
            '=' => {
                self.advance();
                Token::Eq
            }
            '<' => {
                self.advance();
                Token::Lt
            }
            '>' => {
                self.advance();
                Token::Gt
            }
            ':' => {
                self.advance();
                Token::Colon
            }
            ',' => {
                self.advance();
                Token::Comma
            }
            '(' => {
                self.advance();
                Token::LParen
            }
            ')' => {
                self.advance();
                Token::RParen
            }
            '[' => {
                self.advance();
                Token::LBracket
            }
            ']' => {
                self.advance();
                Token::RBracket
            }
            '{' => {
                self.advance();
                Token::LBrace
            }
            '}' => {
                self.advance();
                Token::RBrace
            }
            _ => {
                return Err(LexError {
                    message: format!("unexpected character: {c:?}"),
                    span: Span::new(start, start + c.len_utf8()),
                });
            }
        };

        Ok((token, Span::new(start, self.pos)))
    }

    fn lex_string(&mut self) -> Result<Spanned<Token<'src>>, LexError> {
        let start = self.pos;
        self.advance(); // Skip opening quote

        let content_start = self.pos;
        while self.pos < self.source.len() && self.current() != '"' {
            if self.current() == '\n' {
                return Err(LexError {
                    message: "unterminated string".to_string(),
                    span: Span::new(start, self.pos),
                });
            }
            self.advance();
        }

        if self.pos >= self.source.len() {
            return Err(LexError {
                message: "unterminated string".to_string(),
                span: Span::new(start, self.pos),
            });
        }

        let content = &self.source[content_start..self.pos];
        self.advance(); // Skip closing quote

        Ok((Token::String(content), Span::new(start, self.pos)))
    }

    fn lex_number(&mut self) -> Result<Spanned<Token<'src>>, LexError> {
        let start = self.pos;

        while self.pos < self.source.len() && self.current().is_ascii_digit() {
            self.advance();
        }

        // Decimal part
        if self.current() == '.' && self.peek().is_ascii_digit() {
            self.advance(); // Skip '.'
            while self.pos < self.source.len() && self.current().is_ascii_digit() {
                self.advance();
            }
        }

        let text = &self.source[start..self.pos];
        let value: f64 = text.parse().map_err(|_| LexError {
            message: format!("invalid number: {text}"),
            span: Span::new(start, self.pos),
        })?;

        Ok((Token::Number(value), Span::new(start, self.pos)))
    }

    fn lex_ident(&mut self) -> Spanned<Token<'src>> {
        let start = self.pos;

        while self.pos < self.source.len() {
            let c = self.current();
            if c.is_alphanumeric() || c == '_' || c == '-' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.pos];
        let token = match text.to_lowercase().as_str() {
            "on" => Token::On,
            "if" => Token::If,
            "else" => Token::Else,
            "loop" => Token::Loop,
            "while" => Token::While,
            "set" => Token::Set,
            "to" => Token::To,
            "send" => Token::Send,
            "broadcast" => Token::Broadcast,
            "cancel" => Token::Cancel,
            "stop" => Token::Stop,
            "return" => Token::Return,
            "true" => Token::True,
            "false" => Token::False,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "is" => Token::Is,
            "the" => Token::The,
            "a" => Token::A,
            "an" => Token::An,
            "of" => Token::Of,
            "in" => Token::In,
            "at" => Token::At,
            "from" => Token::From,
            "for" => Token::For,
            "with" => Token::With,
            "without" => Token::Without,
            "command" => Token::Command,
            "trigger" => Token::Trigger,
            "permission" => Token::Permission,
            "aliases" => Token::Aliases,
            "function" => Token::Function,
            _ => Token::Ident(text),
        };

        (token, Span::new(start, self.pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_event() {
        let tokens: Vec<_> = lex("on join:\n\tsend \"Hello\"\n")
            .unwrap()
            .into_iter()
            .map(|(tok, _)| tok)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::On,
                Token::Ident("join"),
                Token::Colon,
                Token::Newline,
                Token::Indent,
                Token::Send,
                Token::String("Hello"),
                Token::Newline,
                Token::Dedent,
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let tokens: Vec<_> = lex("42\n3.25\n")
            .unwrap()
            .into_iter()
            .map(|(tok, _)| tok)
            .collect();

        assert!(matches!(tokens[0], Token::Number(n) if (n - 42.0).abs() < f64::EPSILON));
        assert!(matches!(tokens[2], Token::Number(n) if (n - 3.25).abs() < f64::EPSILON));
    }

    #[test]
    fn test_keywords() {
        let tokens: Vec<_> = lex("if else loop while\n")
            .unwrap()
            .into_iter()
            .map(|(tok, _)| tok)
            .collect();

        assert_eq!(
            tokens[..4],
            [Token::If, Token::Else, Token::Loop, Token::While]
        );
    }

    #[test]
    fn test_comment() {
        let tokens: Vec<_> = lex("on join: # this is a comment\n\tsend \"hi\"\n")
            .unwrap()
            .into_iter()
            .map(|(tok, _)| tok)
            .collect();

        // Comment should be ignored
        assert!(
            !tokens
                .iter()
                .any(|t| matches!(t, Token::Ident(s) if s.contains("comment")))
        );
    }
}
