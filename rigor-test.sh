#! /usr/bin/bash

export CORE_COUNT=2
cargo test || exit
cargo test --release || exit

export CORE_COUNT=4
cargo test || exit
cargo test --release || exit

export CORE_COUNT=8
cargo test || exit
cargo test --release || exit

echo $CORE_COUNT