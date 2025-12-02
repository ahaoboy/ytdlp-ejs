# Improving yt-dlp-ejs with Rust: Smaller and Faster

## Motivation

In the [mpv-easy](https://github.com/mpv-easy/mpv-easy) project, there's a [web tool](https://mpv-easy.github.io/mpv-build) for customizing mpv players and scripts, including yt-dlp. To leverage GitHub CDN (which requires files under 25MB), file size is a constraint. Node/Bun/Deno runtimes are nearly impossible to compress below 25MB, and QuickJS performance in [yt-dlp-ejs](https://github.com/yt-dlp/ejs) is suboptimal.

## The Problem

The [yt-dlp/ejs](https://github.com/yt-dlp/ejs) implementation uses meriyah to parse JavaScript, modifies the AST, regenerates code, and executes it via a JS engine.

Profiling reveals that for lightweight engines like QuickJS and Boa, the bottleneck is code generation. When generating code from a large AST, extensive string concatenation occurs—something these engines don't optimize well. (Note: [QuickJS](https://github.com/quickjs-ng/quickjs/issues/1172#issuecomment-3590109901) recently added optimizations for this scenario.)

In contrast, SWC's code generation is highly efficient. This explains why SWC + QuickJS outperforms Deno/Node/Bun—their CLI startup time is comparable to JS execution time.

```
[TRACE] → readFile
[TRACE] ← readFile (2.14ms)
[TRACE] → main
  [TRACE] → preprocessPlayer
    [TRACE] → parse
    [TRACE] ← parse (3187.57ms)
    [TRACE] → filterExpressions
    [TRACE] ← filterExpressions (251.01ms)
    [TRACE] → generate
```

## Solution

Using Rust, SWC replaces meriyah for AST manipulation. For Deno/Node/Bun, code is saved to a temp file and executed via CLI. For QuickJS/Boa, code is executed directly, eliminating CLI startup overhead.

## Benchmark Results (Ubuntu)

### JavaScript Implementation

Latest results: [bench.yml](https://github.com/ahaoboy/ejs/actions/workflows/bench.yml)

| Runtime | Pass | Fail | Total | Time |
|---------|------|------|-------|------|
| bun | 316 | 0 | 316 | 93.207s |
| node | 316 | 0 | 316 | 90.037s |
| deno | 316 | 0 | 316 | 100.980s |

### Rust Implementation

Latest results: [bench.yml](https://github.com/ahaoboy/ytdlp-ejs/actions/workflows/bench.yml)

| Runtime | Pass | Fail | Total | Time |
|---------|------|------|-------|------|
| qjs | 316 | 0 | 316 | 80.101s |
| node | 316 | 0 | 316 | 87.947s |
| bun | 316 | 0 | 316 | 147.197s |
| boa | 316 | 0 | 316 | 178.557s |
| deno | 316 | 0 | 316 | 238.250s |

## Binary Size Analysis

Using [bloaty-metafile](https://github.com/ahaoboy/bloaty-metafile) for analysis:

- Boa: ~6MB
- SWC: ~2MB
- QuickJS + std::core: ~1MB

![Size breakdown](https://github.com/user-attachments/assets/df36136e-2116-4438-99eb-0be14982d123)

With Boa's `intl_bundled` feature, an additional ~10MB is added. Using SWC + QuickJS keeps the binary under 5MB.

![Size with intl](https://github.com/user-attachments/assets/4d83b56a-908b-470f-bef2-53bfd601d6c8)

## Conclusion

Based on benchmarks, SWC + QuickJS appears to be a compelling alternative—faster execution with a smaller footprint. Would yt-dlp be interested in exploring a Rust-based approach?
