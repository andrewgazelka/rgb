//! Query DSL parser
//!
//! Parses Flecs-like query strings into a structured Query AST.

use std::fmt;

/// A parsed query containing multiple terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub terms: Vec<Term>,
}

impl Query {
    /// Check if this query matches all entities (wildcard only).
    #[must_use]
    pub fn is_wildcard_only(&self) -> bool {
        self.terms.len() == 1 && matches!(self.terms[0].kind, TermKind::Wildcard)
    }

    /// Get all required component names (And operator, not pairs).
    pub fn required_components(&self) -> impl Iterator<Item = &str> {
        self.terms.iter().filter_map(|t| {
            if t.operator == Operator::And {
                t.name()
            } else {
                None
            }
        })
    }

    /// Get all excluded component names (Not operator).
    pub fn excluded_components(&self) -> impl Iterator<Item = &str> {
        self.terms.iter().filter_map(|t| {
            if t.operator == Operator::Not {
                t.name()
            } else {
                None
            }
        })
    }

    /// Get all optional component names (Optional operator).
    pub fn optional_components(&self) -> impl Iterator<Item = &str> {
        self.terms.iter().filter_map(|t| {
            if t.operator == Operator::Optional {
                t.name()
            } else {
                None
            }
        })
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for term in &self.terms {
            if !first {
                if term.operator == Operator::Or {
                    write!(f, " || ")?;
                } else {
                    write!(f, ", ")?;
                }
            }
            first = false;

            match term.operator {
                Operator::Not => write!(f, "!")?,
                Operator::Optional => write!(f, "?")?,
                Operator::And | Operator::Or => {}
            }

            match &term.kind {
                TermKind::Component(name) => write!(f, "{name}")?,
                TermKind::Wildcard => write!(f, "*")?,
                TermKind::Pair(pair) => write!(f, "({}, {})", pair.relation, pair.target)?,
            }
        }
        Ok(())
    }
}

/// A single term in a query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub operator: Operator,
    pub kind: TermKind,
}

impl Term {
    /// Get the component name if this is a Component term.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match &self.kind {
            TermKind::Component(name) => Some(name),
            _ => None,
        }
    }
}

/// The kind of term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermKind {
    /// A component name like "Position"
    Component(String),
    /// A wildcard "*" matching any component
    Wildcard,
    /// A pair like "(ChildOf, Player)"
    Pair(Pair),
}

/// A relationship pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair {
    pub relation: String,
    pub target: String,
}

/// Query operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Operator {
    /// Entity must have the component (default)
    #[default]
    And,
    /// Entity must NOT have the component
    Not,
    /// Component is optional (matched if present)
    Optional,
    /// Either this or previous term must match
    Or,
}

/// Parse error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "parse error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Parse a query string into a Query AST.
///
/// # Syntax
///
/// - `Component` - match entities with Component
/// - `!Component` - match entities WITHOUT Component
/// - `?Component` - optionally match Component
/// - `A || B` - match entities with A OR B
/// - `(Relation, Target)` - match pair relationship
/// - `*` - wildcard, match any
///
/// # Errors
///
/// Returns `ParseError` if the query string is malformed.
pub fn parse_query(input: &str) -> Result<Query, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse()
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse(&mut self) -> Result<Query, ParseError> {
        let mut terms = Vec::new();

        self.skip_whitespace();

        while !self.is_eof() {
            let term = self.parse_term()?;
            terms.push(term);

            self.skip_whitespace();

            if self.is_eof() {
                break;
            }

            // Check for separator
            if self.peek() == Some(',') {
                self.advance();
                self.skip_whitespace();
            } else if self.check_str("||") {
                self.advance();
                self.advance();
                self.skip_whitespace();
                // Next term gets Or operator
                if !self.is_eof() {
                    let mut term = self.parse_term()?;
                    term.operator = Operator::Or;
                    terms.push(term);
                    self.skip_whitespace();
                    if self.peek() == Some(',') {
                        self.advance();
                        self.skip_whitespace();
                    }
                }
            }
        }

        if terms.is_empty() {
            return Err(ParseError {
                message: "empty query".to_string(),
                position: 0,
            });
        }

        Ok(Query { terms })
    }

    fn parse_term(&mut self) -> Result<Term, ParseError> {
        self.skip_whitespace();

        // Check for operator prefix
        let operator = if self.peek() == Some('!') {
            self.advance();
            Operator::Not
        } else if self.peek() == Some('?') {
            self.advance();
            Operator::Optional
        } else {
            Operator::And
        };

        self.skip_whitespace();

        // Check for wildcard
        if self.peek() == Some('*') {
            self.advance();
            return Ok(Term {
                operator,
                kind: TermKind::Wildcard,
            });
        }

        // Check for pair
        if self.peek() == Some('(') {
            let pair = self.parse_pair()?;
            return Ok(Term {
                operator,
                kind: TermKind::Pair(pair),
            });
        }

        // Parse component name
        let name = self.parse_identifier()?;
        Ok(Term {
            operator,
            kind: TermKind::Component(name),
        })
    }

    fn parse_pair(&mut self) -> Result<Pair, ParseError> {
        // Consume '('
        if self.peek() != Some('(') {
            return Err(ParseError {
                message: "expected '('".to_string(),
                position: self.pos,
            });
        }
        self.advance();
        self.skip_whitespace();

        // Parse relation
        let relation = self.parse_identifier()?;

        self.skip_whitespace();

        // Consume ','
        if self.peek() != Some(',') {
            return Err(ParseError {
                message: "expected ',' in pair".to_string(),
                position: self.pos,
            });
        }
        self.advance();
        self.skip_whitespace();

        // Parse target (can be identifier or $variable)
        let target = if self.peek() == Some('$') {
            self.advance();
            format!("${}", self.parse_identifier()?)
        } else {
            self.parse_identifier()?
        };

        self.skip_whitespace();

        // Consume ')'
        if self.peek() != Some(')') {
            return Err(ParseError {
                message: "expected ')' in pair".to_string(),
                position: self.pos,
            });
        }
        self.advance();

        Ok(Pair { relation, target })
    }

    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let mut ident = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if ident.is_empty() {
            return Err(ParseError {
                message: "expected identifier".to_string(),
                position: self.pos,
            });
        }

        Ok(ident)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn check_str(&self, s: &str) -> bool {
        self.remaining().starts_with(s)
    }

    fn remaining(&self) -> &str {
        self.input.get(self.pos..).unwrap_or("")
    }

    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}
