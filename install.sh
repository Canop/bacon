# This script installs bacon for the local system with
#  the most desirable set of features.
#
# If it doesn't compile try removing some feature.
#
# bacon can also be installed system-wide with
#  cargo install --locked .

cargo install --features "clipboard sound" --path .

