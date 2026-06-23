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
# Default build (QuickJS + Boa + External runtimes)
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

### Integration with yt-dlp

Use `--js-runtimes` to plug ejs into yt-dlp as an external JavaScript runtime.
Enable `jsc_trace=true` to see challenge/response pairs in yt-dlp's verbose output.

```bash
 yt-dlp -v --extractor-args "youtube:jsc_trace=true" -F "https://www.youtube.com/watch?v=BnnbP7pCIvQ" --js-runtimes "quickjs:/path/ejs.exe"
```

### As a Library

```rust
use ejs::{
    process_input_with_runtime, JsChallengeInput, JsChallengeRequest,
    JsChallengeType, RuntimeType,
};

let input = JsChallengeInput::Player {
    player: player_code.to_string(),
    requests: vec![
        JsChallengeRequest {
            challenge_type: JsChallengeType::N,
            challenges: vec!["ZdZIqFPQK-Ty8wId".to_string()],
        },
    ],
    output_preprocessed: false,
};

let output = process_input_with_runtime(input, RuntimeType::QuickJS);
```

## Runtime Options

| Runtime | Feature | Binary Size | External Dependency |
|---------|---------|-------------|---------------------|
| QuickJS | `qjs` | ~5MB | None (embedded) |
| Boa | `boa` | ~8MB | None (embedded) |
| Node | `external` | - | Requires Node.js |
| Deno | `external` | - | Requires Deno |
| Bun | `external` | - | Requires Bun |

## Benchmark (Ubuntu)

Latest results from CI ([bench.yml](https://github.com/ahaoboy/ytdlp-ejs/actions/workflows/bench.yml)):

```
========================================
Runtime      Passed   Failed    Total         Time
----------------------------------------
qjs             316        0      316     82.227s
deno            316        0      316    205.101s
boa             316        0      316    508.184s
node            316        0      316    105.107s
bun             316        0      316    136.627s
```

> **qjs** in the table above refers to ejs itself (embedded QuickJS with SWC-based preprocessing).

### Why is ejs faster than other runtimes?

yt-dlp's built-in JSC solver uses [meriyah](https://github.com/meriyah/meriyah) (JS parser)
and [astring](https://github.com/davidbonnet/astring) (JS code generator) to transform
the large YouTube player JavaScript (~2.7MB) — extracting the n/sig solver functions,
generating wrapper code, and executing it.

On lightweight engines like QuickJS, the JS-based code generation step is extremely slow
due to **massive string concatenation** during AST → code output. V8-based engines
(Node, Deno, Bun) have highly optimized string handling and thus perform better on
the pure-JS path, but are still bottlenecked by the meriyah parsing.

This project replaces the entire AST pipeline with **[SWC](https://swc.rs)** (Rust):
parsing, transformation, and code generation all happen in native code. The generated
solver code is then executed via embedded QuickJS. The result:

| Phase | yt-dlp built-in (qjs) | ejs (this project) |
|-------|----------------------|---------------------|
| JS parsing | meriyah (JS) | SWC (Rust) |
| AST transform | meriyah (JS) | SWC (Rust) |
| Code generation | astring (JS) | SWC (Rust) |
| JS execution | QuickJS | QuickJS (embedded) |
| **Total** | ~510s (Boa) | **~82s** |

Even compared to V8 engines (Node, Deno, Bun), ejs wins because SWC's native
preprocessing is faster than meriyah-on-V8, and embedded QuickJS has **zero
process startup overhead** vs spawning a separate runtime.

## Related Projects

- [ytdlp-jsc](https://github.com/ahaoboy/ytdlp-jsc) — YouTube player signature solver using SWC + QuickJS
- [musicfree-tauri](https://github.com/ahaoboy/musicfree-tauri) — MusicFree desktop client built with Tauri
