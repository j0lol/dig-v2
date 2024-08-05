wasm:
    cargo build -r --target wasm32-unknown-unknown
    cp target/wasm32-unknown-unknown/release/another-game.wasm build/

test-wasm:
    cargo build -r --target wasm32-unknown-unknown
    cp target/wasm32-unknown-unknown/release/another-game.wasm build/
    open http://localhost:8000 &
    cd build && php -S localhost:8000
    cd ..

send-wasm:
    cargo build -r --target wasm32-unknown-unknown
    wormhole-rs send target/wasm32-unknown-unknown/release/another-game.wasm