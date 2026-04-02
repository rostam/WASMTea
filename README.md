# WASMTea

A CLI tool that converts Markdown files to HTML, written in Rust and compiled to WASI (WebAssembly System Interface).

## What it does

Takes a Markdown file as input and outputs the rendered HTML to stdout.

## Usage

```bash
wasmtea <filename>
```

**Example:**

```bash
wasmtea example_markdown.md
```

**Output:**

```html
<h1>Hello!</h1>
<p>I am example markdown for this demo!</p>
```

## Building

### Native (Rust)

```bash
cd wasmtea
cargo build --release
```

### WASI (WebAssembly)

Requires the `wasm32-wasip1` target and a WASI runtime (e.g. [Wasmtime](https://wasmtime.dev/)):

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
wasmtime run target/wasm32-wasip1/release/wasmtea.wasm -- example_markdown.md
```

## Dependencies

- [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark) — Markdown parser
- [`structopt`](https://crates.io/crates/structopt) — CLI argument parsing
