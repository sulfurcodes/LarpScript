
# LarpScript

> A programming language for the chronically online.

LarpScript is a small interpreted programming language written in Rust.
It uses a custom slang-inspired syntax and Pest for parsing.

## Features

- Variables and constants
- Numbers, strings, booleans and null
- Arithmetic and comparison operators
- Logical operators
- Operator precedence
- Factorial operator
- Conditional statements
- While-style loops
- Break and continue
- Functions and parameters
- Recursive function calls
- Return values
- Nested scopes
- Runtime and syntax errors
- Single-line comments

## Syntax

| Keyword | Meaning |
|---|---|
| `cook` | Declare a variable |
| `locked` | Declare a constant |
| `yap` | Print |
| `fax` | True |
| `cap` | False |
| `nah` | Null |
| `loop` | Loop |
| `grind` | Loop over a range (`grind i in 5`) or a string's characters (`grind c in "rizz"`) |
| `bet` | Break |
| `clown` | Continue |
| `fun` | Function |
| `mog` | Return |

## Example

```larp
cook username = "Isam";
cook age = 19;

fun factorial(n) {
    if n <= 1 {
        mog 1;
    }

    mog n * factorial(n - 1);
}

yap("Hello, " + username);
yap(factorial(5));

if age >= 18 {
    yap("You're an adult.");
} else {
    yap("You're a minor.");
}
```

## Running LarpScript

Requirements:
- Rust
- Cargo

Clone the repository:

```bash
git clone https://github.com/sulfurcodes/LarpScript.git
cd LarpScript/LarpScript
```

Run a source file:

```bash
cargo run -- examples/hello.larp
```

Run tests:

```bash
cargo test
```

## Operators

| Category | Operators |
|---|---|
| Arithmetic | `+`, `-`, `*`, `/`, `%`, `^` |
| Comparison | `==`, `!=`, `>`, `<`, `>=`, `<=` |
| Logical | `and`, `or`, `not` |
| Unary | `-` |
| Postfix | `!` |

## Built With

- Rust
- Pest

## License

MIT

