//! Skript language parser and AST.
//!
//! This crate provides a parser for the Skript scripting language used in Minecraft servers.
//! It is designed to be modular, separating lexing, AST, and parsing concerns.
//!
//! # Example
//!
//! ```
//! use skript_lang::parse;
//!
//! let source = r#"
//! on join:
//!     send "Hello!" to player
//! "#;
//!
//! let script = parse(source);
//! ```

mod ast;
mod lexer;
mod parser;

pub use ast::*;
pub use lexer::{LexError, Span, Spanned, Token, lex};
pub use parser::parse;
