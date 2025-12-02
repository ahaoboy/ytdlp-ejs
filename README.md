# ejs-rs

YouTube player signature solver implemented in Rust.

A Rust port of [yt-dlp-ejs](ts/README.md), using SWC for JavaScript parsing and multiple runtime options for execution.

## Features

- Parse YouTube player JavaScript code
- Extract and execute signature (`sig`) decryption functions
- Extract and execute throttle parameter (`n`) decryption functions
- Multiple JavaScript runtime support (QuickJS, Deno)
- Cross-platform support (Windows, Linux, macOS)

## Installation

### From Source

```bash
# Default build (with QuickJS)
cargo build --release

# With Deno support
cargo build --release --features deno

# With all features
cargo build --release --features "qjs,deno"
```

The binary will be available at `target/release/ejs-rs`.

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
ejs-rs = { path = ".", features = ["qjs"] }
```

## Usage

### Command Line

```bash
# Basic usage (uses default QuickJS runtime)
ejs-rs <player_file> [n:<challenge>] [sig:<challenge>]

# Specify runtime
ejs-rs --runtime deno <player_file> [n:<challenge>] [sig:<challenge>]

# Examples
ejs-rs player.js n:ZdZIqFPQK-Ty8wId
ejs-rs --runtime qjs player.js sig:gN7a-hudCuAuPH6f...
ejs-rs --runtime deno player.js n:ZdZIqFPQK-Ty8wId sig:gN7a-hudCuAuPH6f...
```

Output is JSON:

```json
{
  "type": "result",
  "responses": [
    {
      "type": "result",
      "data": {
        "ZdZIqFPQK-Ty8wId": "qmtUsIz04xxiNW"
      }
    },
    {
      "type": "result",
      "data": {
        "gN7a-hudCuAuPH6f...": "ttJC2JfQdSswRAIg..."
      }
    }
  ]
}
```

### As a Library

```rust
use ejs::{process_input, process_input_with_runtime, Input, Request, RequestType, RuntimeType};

// Using default runtime
let input = Input::Player {
    player: player_code.to_string(),
    requests: vec![
        Request {
            req_type: RequestType::N,
            challenges: vec!["ZdZIqFPQK-Ty8wId".to_string()],
        },
    ],
    output_preprocessed: false,
};
let output = process_input(input);

// Using specific runtime
let output = process_input_with_runtime(input, RuntimeType::Deno);
```

## Runtime Options

| Runtime | Feature Flag | Description |
|---------|--------------|-------------|
| QuickJS | `qjs` (default) | Embedded JS engine, no external dependencies |
| Deno | `deno` | Uses external Deno process, requires Deno installed |

## Development

### Prerequisites

- Rust 1.70+
- For Deno runtime: [Deno](https://deno.land/) installed
- For testing: player files from YouTube (see below)

### Download Test Players

Download YouTube player files for testing:

```bash
cargo run --example download_players
```

This downloads player files to `ts/src/yt/solver/test/players/`.

### Run Tests

```bash
# Run all tests with default runtime
cargo run --example run_tests

# Run unit tests
cargo test

# Test with Deno runtime
cargo run --features deno --example run_tests
```

### Project Structure

```
src/
├── lib.rs              # Library entry point
├── main.rs             # CLI entry point
├── types.rs            # Type definitions (Input, Output, Request, Response)
├── utils.rs            # Utility functions
├── test_data.rs        # Test cases data
└── solver/
    ├── mod.rs          # Module declarations
    ├── main.rs         # Main processing logic
    ├── solvers.rs      # Player code preprocessing (SWC parsing)
    ├── sig.rs          # Signature function extraction
    ├── n.rs            # N parameter function extraction
    ├── setup.rs        # Browser environment simulation
    └── runtime/
        ├── mod.rs      # Runtime abstraction
        ├── qjs.rs      # QuickJS implementation
        └── deno.rs     # Deno implementation

examples/
├── download_players.rs # Download test player files
├── run_tests.rs        # Run all solver tests
└── single_test.rs      # Test a single player file
```

### Dependencies

- [swc](https://swc.rs/) - Fast JavaScript/TypeScript parser
- [rquickjs](https://github.com/aspect-build/rquickjs) - QuickJS JavaScript engine bindings (optional)
- [serde](https://serde.rs/) - Serialization framework

## How It Works

1. **Parse**: YouTube player JavaScript is parsed into an AST using SWC
2. **Extract**: The AST is analyzed to find signature and n-parameter decryption functions
3. **Preprocess**: A minimal JavaScript bundle is generated with browser environment simulation
4. **Execute**: The preprocessed code is executed in the selected runtime to solve challenges

## Comparison with TypeScript Version

| Feature | TypeScript (ejs) | Rust (ejs-rs) |
|---------|------------------|---------------|
| Parser | meriyah | SWC |
| JS Engine | Native (Node/Deno/Bun) | QuickJS / Deno |
| Code Generator | astring | SWC codegen |
| Binary Size | N/A (requires runtime) | ~5MB standalone |
| Startup Time | ~100ms | ~10ms (QuickJS) |

## License

This code is licensed under [Unlicense](https://unlicense.org/).

## Related Projects

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - A youtube-dl fork with additional features
- [yt-dlp-ejs](ts/README.md) - The original TypeScript implementation
