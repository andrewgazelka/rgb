//! Parser for the Skript language.
//!
//! Uses chumsky to parse tokens into an AST.

// We track positions by UTF-8 character boundaries when parsing interpolated strings,
// so string slicing is safe here.
#![allow(clippy::string_slice)]

use chumsky::{input::ValueInput, prelude::*};

use crate::ast::*;
use crate::lexer::{Token, lex};

/// Chumsky span type (same as ast::Span)
type CSpan = SimpleSpan<usize>;

/// Parse a Skript source string into an AST.
///
/// # Errors
///
/// Returns errors if the source cannot be parsed.
pub fn parse(source: &str) -> Result<Script<'_>, Vec<String>> {
    // First, lex the source
    let tokens = lex(source).map_err(|e| vec![format!("Lex error: {e}")])?;

    // Convert our spans to chumsky spans
    let tokens: Vec<(Token<'_>, CSpan)> = tokens
        .into_iter()
        .map(|(tok, span)| (tok, (span.start..span.end).into()))
        .collect();

    // Then parse
    let len = source.len();
    let eoi: CSpan = (len..len).into();
    let input = tokens.as_slice().map(eoi, |(t, s)| (t, s));

    let (ast, parse_errors) = script_parser().parse(input).into_output_errors();

    if !parse_errors.is_empty() {
        return Err(parse_errors
            .into_iter()
            .map(|e| format!("Parse error: {e}"))
            .collect());
    }

    ast.ok_or_else(|| vec!["Parser succeeded but returned no AST".to_string()])
}

/// Parser for a complete script.
fn script_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Script<'src>, extra::Err<Rich<'tokens, Token<'src>, CSpan>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = CSpan>,
{
    item_parser()
        .repeated()
        .collect::<Vec<_>>()
        .map(|items| Script { items })
}

/// Parser for a top-level item.
fn item_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Item<'src>, extra::Err<Rich<'tokens, Token<'src>, CSpan>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = CSpan>,
{
    // Skip any leading newlines
    let skip_newlines = select! { Token::Newline => () }.repeated();

    skip_newlines.ignore_then(event_parser().map(Item::Event))
}

/// Parser for an event handler: `on <event>:`
fn event_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, EventHandler<'src>, extra::Err<Rich<'tokens, Token<'src>, CSpan>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = CSpan>,
{
    let event_name = select! { Token::Ident(s) => s }.labelled("event name");

    just(Token::On)
        .ignore_then(event_name)
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline))
        .then(block_parser())
        .map_with(|(event, body), e| EventHandler {
            event,
            body,
            span: e.span(),
        })
        .labelled("event handler")
}

/// Parser for a block (indented section).
fn block_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Block<'src>, extra::Err<Rich<'tokens, Token<'src>, CSpan>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = CSpan>,
{
    just(Token::Indent)
        .ignore_then(
            simple_stmt_parser()
                .then_ignore(just(Token::Newline).or_not())
                .repeated()
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::Dedent))
        .map_with(|stmts, e| Block {
            stmts,
            span: e.span(),
        })
        .labelled("block")
}

/// Parser for a simple statement (no nesting).
fn simple_stmt_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Stmt<'src>, extra::Err<Rich<'tokens, Token<'src>, CSpan>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = CSpan>,
{
    // Send effect: send <message> [to <target>]
    let send_effect = just(Token::Send)
        .ignore_then(expr_parser())
        .then(just(Token::To).ignore_then(expr_parser()).or_not())
        .map_with(|(message, target), e| {
            Stmt::Effect(Effect {
                kind: EffectKind::Send {
                    message: Box::new(message),
                    target: target.map(Box::new),
                },
                span: e.span(),
            })
        });

    // Broadcast effect: broadcast <message>
    let broadcast_effect =
        just(Token::Broadcast)
            .ignore_then(expr_parser())
            .map_with(|message, e| {
                Stmt::Effect(Effect {
                    kind: EffectKind::Broadcast {
                        message: Box::new(message),
                    },
                    span: e.span(),
                })
            });

    // Cancel effect: cancel [the] event
    let cancel_effect = just(Token::Cancel)
        .ignore_then(just(Token::The).or_not())
        .ignore_then(select! { Token::Ident("event") => () }.or_not())
        .map_with(|_, e| {
            Stmt::Effect(Effect {
                kind: EffectKind::Cancel,
                span: e.span(),
            })
        });

    // Set statement: set <target> to <value>
    let set_stmt = just(Token::Set)
        .ignore_then(expr_parser())
        .then_ignore(just(Token::To))
        .then(expr_parser())
        .map_with(|(target, value), e| {
            Stmt::Set(SetStmt {
                target,
                value,
                span: e.span(),
            })
        });

    // Stop statement
    let stop_stmt = just(Token::Stop).map_with(|_, e| Stmt::Stop(e.span()));

    // Return statement
    let return_stmt = just(Token::Return)
        .ignore_then(expr_parser().or_not())
        .map_with(|expr, e| Stmt::Return(expr, e.span()));

    // Condition as statement (guard) - simplified
    let condition_stmt = simple_condition_parser().map(Stmt::Condition);

    // Expression statement
    let expr_stmt = expr_parser().map(Stmt::Expr);

    choice((
        send_effect,
        broadcast_effect,
        cancel_effect,
        set_stmt,
        stop_stmt,
        return_stmt,
        condition_stmt,
        expr_stmt,
    ))
    .labelled("statement")
}

/// Simplified condition parser (no nesting).
fn simple_condition_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Condition<'src>, extra::Err<Rich<'tokens, Token<'src>, CSpan>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = CSpan>,
{
    let expr = expr_parser();

    // <expr> is [not] <expr>
    expr.clone()
        .then_ignore(just(Token::Is))
        .then(just(Token::Not).or_not().map(|n| n.is_some()).then(expr))
        .map_with(|(left, (negated, right)), e| Condition {
            kind: if negated {
                ConditionKind::IsNot(Box::new(left), Box::new(right))
            } else {
                ConditionKind::Is(Box::new(left), Box::new(right))
            },
            negated: false,
            span: e.span(),
        })
        .labelled("condition")
}

/// Parser for an expression.
fn expr_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr<'src>, extra::Err<Rich<'tokens, Token<'src>, CSpan>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = CSpan>,
{
    // Literals
    let number = select! { Token::Number(n) => n }.map_with(|n, e| {
        Expr::Literal(Literal {
            kind: LiteralKind::Number(n),
            span: e.span(),
        })
    });

    let string = select! { Token::String(s) => s }.map_with(|s, e| {
        // Check for interpolation markers %
        if s.contains('%') {
            let parts = parse_interpolated_string(s, e.span());
            Expr::InterpolatedString {
                parts,
                span: e.span(),
            }
        } else {
            Expr::Literal(Literal {
                kind: LiteralKind::String(s),
                span: e.span(),
            })
        }
    });

    let boolean =
        choice((just(Token::True).to(true), just(Token::False).to(false))).map_with(|b, e| {
            Expr::Literal(Literal {
                kind: LiteralKind::Boolean(b),
                span: e.span(),
            })
        });

    // Variable: {name} or {_local}
    let variable = just(Token::LBrace)
        .ignore_then(select! { Token::Ident(s) => s })
        .then_ignore(just(Token::RBrace))
        .map_with(|name, e| {
            let local = name.starts_with('_');
            Expr::Variable(Variable {
                name,
                local,
                indices: vec![],
                span: e.span(),
            })
        });

    // Identifier
    let ident = select! { Token::Ident(s) => s }.map_with(|s, e| Expr::Ident(s, e.span()));

    // "the" prefix is optional
    let the_expr = just(Token::The).ignore_then(ident).or(ident);

    // Atom: basic expression
    choice((number, string, boolean, variable, the_expr)).labelled("expression")
}

/// Parse an interpolated string like "Hello %player%!"
fn parse_interpolated_string<'src>(s: &'src str, _span: CSpan) -> Vec<StringPart<'src>> {
    let mut parts = Vec::new();
    let mut current_start = 0;
    let mut in_expr = false;
    let mut expr_start = 0;

    for (i, c) in s.char_indices() {
        if c == '%' {
            if in_expr {
                // End of expression
                let expr_str = &s[expr_start..i];
                parts.push(StringPart::Expr(Expr::Ident(
                    expr_str,
                    (expr_start..i).into(),
                )));
                in_expr = false;
                current_start = i + 1;
            } else {
                // Start of expression
                if current_start < i {
                    parts.push(StringPart::Literal(&s[current_start..i]));
                }
                in_expr = true;
                expr_start = i + 1;
            }
        }
    }

    // Handle remaining literal
    if current_start < s.len() && !in_expr {
        parts.push(StringPart::Literal(&s[current_start..]));
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_event() {
        let source = "on join:\n\tsend \"Hello\"\n";
        let result = parse(source);
        assert!(result.is_ok(), "Parse failed: {result:?}");

        let script = result.unwrap();
        assert_eq!(script.items.len(), 1);

        match &script.items[0] {
            Item::Event(e) => {
                assert_eq!(e.event, "join");
                assert_eq!(e.body.stmts.len(), 1);
            }
            _ => panic!("Expected event"),
        }
    }

    #[test]
    fn test_parse_event_with_target() {
        let source = "on join:\n\tsend \"Hello\" to player\n";
        let result = parse(source);
        assert!(result.is_ok(), "Parse failed: {result:?}");
    }

    #[test]
    fn test_parse_interpolated_string() {
        let parts = parse_interpolated_string("Hello %player%!", (0..0).into());
        assert_eq!(parts.len(), 3);
        assert!(matches!(parts[0], StringPart::Literal("Hello ")));
        assert!(matches!(parts[1], StringPart::Expr(_)));
        assert!(matches!(parts[2], StringPart::Literal("!")));
    }
}
