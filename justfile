

wasm:
    cargo build -r --target wasm32-unknown-unknown
    wormhole-rs send target/wasm32-unknown-unknown/release/another-game.wasm