#!/usr/bin/env bash
set -e

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT/wasmtea"

usage() {
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  build         Build native binary"
    echo "  build-wasi    Build WASI/WebAssembly target"
    echo "  run <file>    Run native binary with a markdown file"
    echo "  run-wasi <file>  Run WASI binary with wasmtime"
    echo "  setup-wasi    Add wasm32-wasip1 rustup target"
    echo "  clean         Clean build artifacts"
    echo "  build-web     Build browser WASM via wasm-pack (outputs to wasmtea/pkg/)"
    echo "  build-graph   Build graph editor WASM via wasm-pack (outputs to graph/pkg/)"
    echo "  serve         Serve all HTML files locally on port 8080"
    echo "  all           Setup WASI target, build native, WASI, web, and graph"
    exit 1
}

case "${1}" in
    build)
        cargo build --release
        echo "Built: target/release/wasmtea"
        ;;
    build-wasi)
        cargo build --target wasm32-wasip1 --release
        echo "Built: target/wasm32-wasip1/release/wasmtea.wasm"
        ;;
    run)
        [ -z "$2" ] && { echo "Error: provide a markdown file"; exit 1; }
        cargo run --release -- "$2"
        ;;
    run-wasi)
        [ -z "$2" ] && { echo "Error: provide a markdown file"; exit 1; }
        wasmtime run target/wasm32-wasip1/release/wasmtea.wasm -- "$2"
        ;;
    setup-wasi)
        rustup target add wasm32-wasip1
        ;;
    build-web)
        wasm-pack build --target web --features web
        echo "Built: wasmtea/pkg/"
        ;;
    build-graph)
        cd "$ROOT/graph"
        wasm-pack build --target web
        echo "Built: graph/pkg/"
        ;;
    serve)
        cd "$ROOT"
        echo "Serving at http://localhost:8080"
        python3 -m http.server 8080
        ;;
    clean)
        cargo clean
        rm -rf pkg/
        ;;
    all)
        rustup target add wasm32-wasip1
        cargo build --release
        cargo build --target wasm32-wasip1 --release
        wasm-pack build --target web --features web
        cd "$ROOT/graph"
        wasm-pack build --target web
        echo "Done. Outputs:"
        echo "  target/release/wasmtea                     (native CLI)"
        echo "  target/wasm32-wasip1/release/wasmtea.wasm  (WASI)"
        echo "  wasmtea/pkg/                               (markdown renderer WASM)"
        echo "  graph/pkg/                                 (graph editor WASM)"
        echo ""
        echo "Serve with: ./scripts.sh serve"
        ;;
    *)
        usage
        ;;
esac
