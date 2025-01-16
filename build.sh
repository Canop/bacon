# This script compiles bacon for the local system with
#  the most desirable set of features.
#
# After compilation, bacon can be found in target/release
#
# If it doesn't compile try removing some feature.
#
# bacon can also be installed system-wide with
#  cargo install --locked .
cargo build --release --features "clipboard"

