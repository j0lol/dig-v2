wasm:
    ./wasm-build.sh --release another-game

test-wasm:
    open http://localhost:8000 &
    cd dist && php -S localhost:8000
    cd ..

send-wasm:
    cargo build -r --target wasm32-unknown-unknown
    wormhole-rs send target/wasm32-unknown-unknown/release/another-game.wasm