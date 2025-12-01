//! Flecs-like Query DSL Parser
//!
//! A string-based query language for the RGB ECS, inspired by Flecs Query Language.
//!
//! # Syntax
//!
//! ```text
//! Position, Velocity           // Match entities with both components
//! Position, !Velocity          // Match entities with Position but NOT Velocity
//! Position, ?Velocity          // Match entities with Position, optionally Velocity
//! *                            // Match all entities (list all components)
//! Position || Velocity         // Match entities with Position OR Velocity
//! (ChildOf, $parent)           // Match pair relationships
//! ```
//!
//! # Examples
//!
//! ```
//! use query_dsl::{parse_query, Term, Operator};
//!
//! let query = parse_query("Position, !Velocity, ?Health").unwrap();
//! assert_eq!(query.terms.len(), 3);
//! assert_eq!(query.terms[0].operator, Operator::And);
//! assert_eq!(query.terms[1].operator, Operator::Not);
//! assert_eq!(query.terms[2].operator, Operator::Optional);
//! ```

mod parser;

pub use parser::{Operator, Pair, Query, Term, TermKind, parse_query};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let query = parse_query("Position, Velocity").unwrap();
        assert_eq!(query.terms.len(), 2);
        assert_eq!(query.terms[0].name(), Some("Position"));
        assert_eq!(query.terms[1].name(), Some("Velocity"));
    }

    #[test]
    fn test_not_operator() {
        let query = parse_query("Position, !Velocity").unwrap();
        assert_eq!(query.terms[0].operator, Operator::And);
        assert_eq!(query.terms[1].operator, Operator::Not);
    }

    #[test]
    fn test_optional_operator() {
        let query = parse_query("Position, ?Health").unwrap();
        assert_eq!(query.terms[1].operator, Operator::Optional);
    }

    #[test]
    fn test_wildcard() {
        let query = parse_query("*").unwrap();
        assert_eq!(query.terms.len(), 1);
        assert!(matches!(query.terms[0].kind, TermKind::Wildcard));
    }

    #[test]
    fn test_or_operator() {
        let query = parse_query("Position || Velocity").unwrap();
        assert_eq!(query.terms.len(), 2);
        assert_eq!(query.terms[1].operator, Operator::Or);
    }

    #[test]
    fn test_pair() {
        let query = parse_query("(ChildOf, Player)").unwrap();
        assert_eq!(query.terms.len(), 1);
        if let TermKind::Pair(pair) = &query.terms[0].kind {
            assert_eq!(pair.relation, "ChildOf");
            assert_eq!(pair.target, "Player");
        } else {
            panic!("Expected pair");
        }
    }

    #[test]
    fn test_complex_query() {
        let query = parse_query("Player, Position, !Dead, ?Health").unwrap();
        assert_eq!(query.terms.len(), 4);
        assert_eq!(query.terms[0].name(), Some("Player"));
        assert_eq!(query.terms[1].name(), Some("Position"));
        assert_eq!(query.terms[2].operator, Operator::Not);
        assert_eq!(query.terms[3].operator, Operator::Optional);
    }

    #[test]
    fn test_whitespace_handling() {
        let query = parse_query("  Position  ,  Velocity  ").unwrap();
        assert_eq!(query.terms.len(), 2);
        assert_eq!(query.terms[0].name(), Some("Position"));
        assert_eq!(query.terms[1].name(), Some("Velocity"));
    }
}
