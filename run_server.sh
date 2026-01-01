spacetime generate -p server -l rust -o ./syncer/src/networking
spacetime publish morph -p server
cargo run -p syncer --release