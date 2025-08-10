For WASM:
trunk build --release

# or manually:

cargo build --target wasm32-unknown-unknown --release

# then,

python3 serve.py

# to serve to the html

For native:
cargo run

# or

cargo build --release
