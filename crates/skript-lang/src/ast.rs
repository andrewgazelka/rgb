//! Abstract Syntax Tree for the Skript language.
//!
//! The AST is designed to be:
//! - Simple and clean
//! - Separate from runtime/execution concerns
//! - Easy to traverse and transform

/// A span in the source code.
pub type Span = chumsky::span::SimpleSpan<usize>;

/// A Skript source file containing multiple top-level structures.
#[derive(Debug, Clone, PartialEq)]
pub struct Script<'src> {
    pub items: Vec<Item<'src>>,
}

/// A top-level item in a Skript file.
#[derive(Debug, Clone, PartialEq)]
pub enum Item<'src> {
    /// An event handler: `on <event>:`
    Event(EventHandler<'src>),
    /// A command definition: `command /<name>:`
    Command(CommandDef<'src>),
    /// A function definition: `function <name>(<args>):`
    Function(FunctionDef<'src>),
    /// An alias definition: `aliases:`
    Aliases(AliasesDef<'src>),
}

/// An event handler.
#[derive(Debug, Clone, PartialEq)]
pub struct EventHandler<'src> {
    pub event: &'src str,
    pub body: Block<'src>,
    pub span: Span,
}

/// A command definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandDef<'src> {
    pub name: &'src str,
    pub args: Vec<CommandArg<'src>>,
    pub options: Vec<CommandOption<'src>>,
    pub trigger: Block<'src>,
    pub span: Span,
}

/// A command argument like `<text>` or `[<number>]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandArg<'src> {
    pub name: &'src str,
    pub optional: bool,
    pub span: Span,
}

/// A command option like `permission: skript.admin`.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandOption<'src> {
    pub key: &'src str,
    pub value: Expr<'src>,
    pub span: Span,
}

/// A function definition.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef<'src> {
    pub name: &'src str,
    pub params: Vec<Param<'src>>,
    pub return_type: Option<&'src str>,
    pub body: Block<'src>,
    pub span: Span,
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Param<'src> {
    pub name: &'src str,
    pub ty: Option<&'src str>,
    pub default: Option<Expr<'src>>,
    pub span: Span,
}

/// An aliases definition block.
#[derive(Debug, Clone, PartialEq)]
pub struct AliasesDef<'src> {
    pub aliases: Vec<AliasEntry<'src>>,
    pub span: Span,
}

/// A single alias entry: `name = items`.
#[derive(Debug, Clone, PartialEq)]
pub struct AliasEntry<'src> {
    pub name: &'src str,
    pub items: Vec<Expr<'src>>,
    pub span: Span,
}

/// A block of statements (indented section).
#[derive(Debug, Clone, PartialEq)]
pub struct Block<'src> {
    pub stmts: Vec<Stmt<'src>>,
    pub span: Span,
}

/// A statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'src> {
    /// An effect (action): `send "Hello" to player`
    Effect(Effect<'src>),
    /// A condition used as a statement (guard): `player is alive`
    Condition(Condition<'src>),
    /// An if statement
    If(IfStmt<'src>),
    /// A loop statement
    Loop(LoopStmt<'src>),
    /// A while loop
    While(WhileStmt<'src>),
    /// Set statement: `set {var} to value`
    Set(SetStmt<'src>),
    /// Return statement
    Return(Option<Expr<'src>>, Span),
    /// Stop/exit statement
    Stop(Span),
    /// Continue loop
    Continue(Span),
    /// Expression statement (for function calls, etc.)
    Expr(Expr<'src>),
}

/// An effect (action that does something).
#[derive(Debug, Clone, PartialEq)]
pub struct Effect<'src> {
    pub kind: EffectKind<'src>,
    pub span: Span,
}

/// Different kinds of effects.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectKind<'src> {
    /// `send <message> [to <target>]`
    Send {
        message: Box<Expr<'src>>,
        target: Option<Box<Expr<'src>>>,
    },
    /// `broadcast <message>`
    Broadcast { message: Box<Expr<'src>> },
    /// `cancel [the] event`
    Cancel,
    /// `teleport <entity> to <location>`
    Teleport {
        entity: Box<Expr<'src>>,
        location: Box<Expr<'src>>,
    },
    /// `give <item> to <player>`
    Give {
        item: Box<Expr<'src>>,
        target: Box<Expr<'src>>,
    },
    /// `delete <expr>`
    Delete { target: Box<Expr<'src>> },
    /// Generic effect pattern: `<pattern>`
    Generic { pattern: &'src str },
}

/// A condition (boolean expression).
#[derive(Debug, Clone, PartialEq)]
pub struct Condition<'src> {
    pub kind: ConditionKind<'src>,
    pub negated: bool,
    pub span: Span,
}

/// Different kinds of conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionKind<'src> {
    /// `<expr> is <expr>`
    Is(Box<Expr<'src>>, Box<Expr<'src>>),
    /// `<expr> is not <expr>`
    IsNot(Box<Expr<'src>>, Box<Expr<'src>>),
    /// `<expr> contains <expr>`
    Contains(Box<Expr<'src>>, Box<Expr<'src>>),
    /// `<entity> has permission <perm>`
    HasPermission(Box<Expr<'src>>, Box<Expr<'src>>),
    /// `<expr> is set`
    IsSet(Box<Expr<'src>>),
    /// `<expr> exists`
    Exists(Box<Expr<'src>>),
    /// Comparison operators
    Compare {
        left: Box<Expr<'src>>,
        op: CompareOp,
        right: Box<Expr<'src>>,
    },
    /// Boolean expression
    Expr(Box<Expr<'src>>),
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
}

/// An if statement.
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt<'src> {
    pub condition: Condition<'src>,
    pub then_block: Block<'src>,
    pub else_block: Option<Block<'src>>,
    pub span: Span,
}

/// A loop statement.
#[derive(Debug, Clone, PartialEq)]
pub struct LoopStmt<'src> {
    pub kind: LoopKind<'src>,
    pub body: Block<'src>,
    pub span: Span,
}

/// Different kinds of loops.
#[derive(Debug, Clone, PartialEq)]
pub enum LoopKind<'src> {
    /// `loop <n> times`
    Times(Box<Expr<'src>>),
    /// `loop <expr>` (iterate over collection)
    Each(Box<Expr<'src>>),
}

/// A while loop.
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt<'src> {
    pub condition: Condition<'src>,
    pub body: Block<'src>,
    pub do_while: bool, // true for `do while`
    pub span: Span,
}

/// A set statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SetStmt<'src> {
    pub target: Expr<'src>,
    pub value: Expr<'src>,
    pub span: Span,
}

/// An expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'src> {
    /// A literal value
    Literal(Literal<'src>),
    /// A variable: `{name}` or `{_local}`
    Variable(Variable<'src>),
    /// An identifier (event values, etc.): `player`, `event-block`
    Ident(&'src str, Span),
    /// Binary operation
    Binary {
        left: Box<Self>,
        op: BinaryOp,
        right: Box<Self>,
        span: Span,
    },
    /// Unary operation
    Unary {
        op: UnaryOp,
        expr: Box<Self>,
        span: Span,
    },
    /// Function call: `func(args)`
    Call {
        name: &'src str,
        args: Vec<Self>,
        span: Span,
    },
    /// Property access: `<expr>'s <property>` or `<property> of <expr>`
    Property {
        object: Box<Self>,
        property: &'src str,
        span: Span,
    },
    /// Index access: `<expr>::<index>` or `<expr>[<index>]`
    Index {
        object: Box<Self>,
        index: Box<Self>,
        span: Span,
    },
    /// String with interpolation: `"Hello %player%"`
    InterpolatedString {
        parts: Vec<StringPart<'src>>,
        span: Span,
    },
    /// List literal: `1, 2, and 3`
    List { items: Vec<Self>, span: Span },
    /// Conditional expression
    Conditional {
        condition: Box<Condition<'src>>,
        then_expr: Box<Self>,
        else_expr: Box<Self>,
        span: Span,
    },
}

impl<'src> Expr<'src> {
    /// Get the span of this expression.
    pub fn span(&self) -> Span {
        match self {
            Self::Literal(lit) => lit.span,
            Self::Variable(var) => var.span,
            Self::Ident(_, span)
            | Self::Binary { span, .. }
            | Self::Unary { span, .. }
            | Self::Call { span, .. }
            | Self::Property { span, .. }
            | Self::Index { span, .. }
            | Self::InterpolatedString { span, .. }
            | Self::List { span, .. }
            | Self::Conditional { span, .. } => *span,
        }
    }
}

/// A literal value.
#[derive(Debug, Clone, PartialEq)]
pub struct Literal<'src> {
    pub kind: LiteralKind<'src>,
    pub span: Span,
}

/// Kinds of literal values.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind<'src> {
    Number(f64),
    String(&'src str),
    Boolean(bool),
}

/// A variable reference.
#[derive(Debug, Clone, PartialEq)]
pub struct Variable<'src> {
    pub name: &'src str,
    pub local: bool,              // {_name} vs {name}
    pub indices: Vec<Expr<'src>>, // for {var::index::*}
    pub span: Span,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Part of an interpolated string.
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart<'src> {
    Literal(&'src str),
    Expr(Expr<'src>),
}
