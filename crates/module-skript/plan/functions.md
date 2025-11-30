# Functions

User-defined functions with parameters and return values.

## Syntax

```skript
function greet(player: player):
    send "Hello!" to player

function add(a: number, b: number) :: number:
    return a + b

function getPlayers() :: players:
    return all players
```

## Definition

```skript
function <name>([<params>]) [:: <return type>]:
    <statements>
```

### Parameters

```skript
function example(
    required: text,           # required parameter
    optional: number = 5,     # default value
    variadic: texts           # multiple values (must be last)
):
```

### Return Types

| Type | Pattern |
|------|---------|
| Single | `:: number`, `:: player` |
| Multiple | `:: numbers`, `:: players` |
| Optional | No `::` clause |

## Calling Functions

```skript
greet(player)
set {_sum} to add(1, 2)
set {_players::*} to getPlayers()
```

## Implementation

### AST

From `skript-lang`:
```rust
pub struct FunctionDef<'src> {
    pub name: &'src str,
    pub params: Vec<Param<'src>>,
    pub return_type: Option<&'src str>,
    pub body: Block<'src>,
    pub span: Span,
}

pub struct Param<'src> {
    pub name: &'src str,
    pub ty: Option<&'src str>,
    pub default: Option<Expr<'src>>,
    pub span: Span,
}
```

### Runtime

```rust
pub struct Function {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<ValueType>,
    pub body: CompiledBlock,
}

pub struct FunctionParam {
    pub name: String,
    pub ty: ValueType,
    pub default: Option<Value>,
}

pub struct FunctionRegistry {
    functions: HashMap<String, Function>,
}

impl FunctionRegistry {
    pub fn call(
        &self,
        name: &str,
        args: Vec<Value>,
        ctx: &mut ExecutionContext,
    ) -> Value {
        let func = self.functions.get(name)?;

        // Create local scope for function
        let mut local_vars = HashMap::new();

        // Bind parameters
        for (i, param) in func.params.iter().enumerate() {
            let value = args.get(i)
                .cloned()
                .or_else(|| param.default.clone())
                .unwrap_or(Value::None);
            local_vars.insert(param.name.clone(), value);
        }

        // Execute with new scope
        let mut func_ctx = ctx.with_locals(local_vars);
        match execute_block(&func.body, &mut func_ctx) {
            EffectResult::Return(value) => value,
            _ => Value::None,
        }
    }
}
```

### Call Expression

```rust
// In expr evaluation
Expr::Call { name, args, .. } => {
    let evaluated_args: Vec<Value> = args
        .iter()
        .map(|arg| evaluate_expr(arg, ctx))
        .collect();

    ctx.function_registry.call(name, evaluated_args, ctx)
}
```

## Local Functions

Functions can be defined within scripts:

```skript
on load:
    # Functions defined at top of script are available here

function helper():
    # Available to all triggers in this script
```

## Recursion

Functions can call themselves:

```skript
function factorial(n: number) :: number:
    if n <= 1:
        return 1
    return n * factorial(n - 1)
```

Stack depth limit to prevent infinite recursion:

```rust
const MAX_CALL_DEPTH: usize = 100;

impl ExecutionContext {
    pub fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, Error> {
        if self.call_depth >= MAX_CALL_DEPTH {
            return Err(Error::StackOverflow);
        }
        self.call_depth += 1;
        let result = self.function_registry.call(name, args, self);
        self.call_depth -= 1;
        Ok(result)
    }
}
```
