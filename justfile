run:
    cargo run

wasm:
    ./wasm-build.sh --release another-game

test-wasm:
    just wasm
    open http://localhost:8000 &
    cd dist && php -S localhost:8000
    cd ..