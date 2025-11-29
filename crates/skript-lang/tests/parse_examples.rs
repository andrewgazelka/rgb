//! Integration tests that parse example Skript files.

use skript_lang::parse;

const SIMPLE_EVENT: &str = include_str!("examples/simple_event.sk");
const BROADCAST: &str = include_str!("examples/broadcast.sk");
const CONDITIONAL: &str = include_str!("examples/conditional.sk");

#[test]
fn test_simple_event() {
    let result = parse(SIMPLE_EVENT);
    assert!(
        result.is_ok(),
        "Failed to parse simple_event.sk: {result:?}"
    );

    let script = result.unwrap();
    assert_eq!(script.items.len(), 1);
}

#[test]
fn test_broadcast() {
    let result = parse(BROADCAST);
    assert!(result.is_ok(), "Failed to parse broadcast.sk: {result:?}");
}

#[test]
fn test_conditional() {
    let result = parse(CONDITIONAL);
    assert!(result.is_ok(), "Failed to parse conditional.sk: {result:?}");
}
