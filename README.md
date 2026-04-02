# WASMTea

A browser-based graph editing and analysis framework built with Rust and WebAssembly.

The graph engine runs entirely in the browser — no server, no backend. The Rust core is compiled to WASM via `wasm-pack` and communicates with a canvas-based UI through `wasm-bindgen`.

> `index.html` will serve as the introduction and landing page for the software.

## Features

### Editor
- Add vertices by clicking the canvas
- Draw edges by dragging between vertices
- Move vertices by dragging
- Delete vertices or edges with a click

### Analysis sidebar
Run graph algorithms on any graph you build:

| Category | Algorithms |
|---|---|
| Basic | Vertex/edge count, density |
| Connectivity | Connected components, diameter |
| Degrees | Min / max / avg degree, degree sequence |
| Properties | Bipartite check, Eulerian path / circuit |
| Coloring | Greedy chromatic number |
| NP-Hard (exact) | Maximum independent set, minimum vertex cover, maximum clique |

Exact NP-hard algorithms use backtracking and run for graphs up to 25 vertices.

## Project structure

```
graph/          Rust crate — graph data structure + algorithms (compiled to WASM)
wasmtea/        Rust crate — Markdown-to-HTML CLI (WASI demo)
graph.html      Graph editor UI
index.html      Introduction / landing page (in progress)
scripts.sh      Build and serve commands
```

## Getting started

### Requirements

- [Rust](https://rustup.rs/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) — `cargo install wasm-pack --locked`

### Build and run

```bash
# Build the graph editor WASM module
./scripts.sh build-graph

# Serve locally
./scripts.sh serve
# → open http://localhost:8080/graph.html
```

### Build everything

```bash
./scripts.sh all
```

## Scripts reference

```
./scripts.sh build-graph   Build graph editor WASM (outputs to graph/pkg/)
./scripts.sh build-web     Build markdown renderer WASM (outputs to wasmtea/pkg/)
./scripts.sh build         Build native wasmtea CLI
./scripts.sh build-wasi    Build wasmtea for WASI target
./scripts.sh serve         Serve all pages at http://localhost:8080
./scripts.sh run <file>    Run wasmtea CLI on a markdown file
./scripts.sh setup-wasi    Add wasm32-wasip1 rustup target
./scripts.sh clean         Remove build artifacts
./scripts.sh all           Build everything
```

## Tech stack

- **Rust** — graph engine and algorithms
- **wasm-bindgen / wasm-pack** — Rust → WebAssembly + JS bindings
- **HTML5 Canvas** — rendering
- **serde_json** — data serialization between Rust and JS
