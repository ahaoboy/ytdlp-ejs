# ytdlp-ejs

YouTube player signature solver implemented in Rust, using SWC for JavaScript parsing and multiple runtime options for execution.

A Rust port of [yt-dlp/ejs](https://github.com/yt-dlp/ejs).

📖 [Improving yt-dlp-ejs with Rust: Smaller and Faster](blog.md)

## Features

- Parse YouTube player JavaScript code
- Extract and execute signature (`sig`) decryption functions
- Extract and execute throttle parameter (`n`) decryption functions
- Multiple JavaScript runtime support: QuickJS, Boa, Node, Deno, Bun
- Cross-platform support (Windows, Linux, macOS)
- Standalone binary under 5MB (with SWC + QuickJS)

## Installation

### From Source

```bash
# Default build (QuickJS + Boa)
cargo build --release

# QuickJS only (smallest binary)
cargo build --release --no-default-features --features qjs

# Boa only
cargo build --release --no-default-features --features boa
```

Binary output: `target/release/ejs`

### As a Library

```toml
[dependencies]
ejs = { git = "https://github.com/ahaoboy/ytdlp-ejs", features = ["qjs"] }
```

## Usage

### Command Line

```bash
# Basic usage
ejs <player_file> [n:<challenge>] [sig:<challenge>]

# Specify runtime
ejs --runtime qjs player.js n:ZdZIqFPQK-Ty8wId
ejs --runtime boa player.js sig:gN7a-hudCuAuPH6f...
ejs --runtime node player.js n:ZdZIqFPQK-Ty8wId sig:gN7a-hudCuAuPH6f...
ejs --runtime deno player.js n:ZdZIqFPQK-Ty8wId
ejs --runtime bun player.js n:ZdZIqFPQK-Ty8wId
```

Output (JSON):

```json
{
  "type": "result",
  "responses": [
    { "type": "result", "data": { "ZdZIqFPQK-Ty8wId": "qmtUsIz04xxiNW" } }
  ]
}
```

### As a Library

```rust
use ejs::{process_input_with_runtime, Input, Request, RequestType, RuntimeType};

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

let output = process_input_with_runtime(input, RuntimeType::Qjs);
```

## Runtime Options

| Runtime | Feature | Binary Size | External Dependency |
|---------|---------|-------------|---------------------|
| QuickJS | `qjs` | ~5MB | None (embedded) |
| Boa | `boa` | ~8MB | None (embedded) |
| Node | - | - | Requires Node.js |
| Deno | - | - | Requires Deno |
| Bun | - | - | Requires Bun |

## Benchmark (Ubuntu)

| Runtime | Pass | Fail | Total | Time |
|---------|------|------|-------|------|
| qjs | 316 | 0 | 316 | 80.101s |
| node | 316 | 0 | 316 | 87.947s |
| bun | 316 | 0 | 316 | 147.197s |
| boa | 316 | 0 | 316 | 178.557s |
| deno | 316 | 0 | 316 | 238.250s |

Latest results: [bench.yml](https://github.com/ahaoboy/ytdlp-ejs/actions/workflows/bench.yml)

