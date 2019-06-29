#!/usr/bin/env bash

# Set default_features = false for reqwest dependency before building.
# This way we can skip OpenSSL during cross build and use rusttls which
# is far easier to build. We cannot disable default features of dependencies
# on the commandline nor can we switch features based on OS. So for now this
# manual step is necessary.

# For macOS and Windows we still want default features because of the
# superior operating system integration (using the users certificate store).

# Do not check in the resulting Cargo.lock.

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TARGET="${DIR}/../target"
NAME="fritzlogger"

# $1 = llvm target
# $2 = CROSS_COMPILE for C toolchain
build_cross()
{
    local prefix=$2-
    local src="$TARGET/$1/release/$NAME"
    local dest="$TARGET/${NAME}.$1"

    echo "Building $2..."

    CC=${prefix}gcc \
    AR=${prefix}gcc-ar \
    RANLIB=${prefix}gcc-ranlib \
    LD=${prefix}gcc \
    cargo build --target=$1 --release --features reqwest/rustls-tls --manifest-path="$DIR/../Cargo.toml"
    cp "$src" "$dest"
    ${prefix}strip "$dest"
}

build_cross arm-unknown-linux-musleabihf arm-linux-musleabihf 
build_cross aarch64-unknown-linux-musl aarch64-linux-musl 
build_cross x86_64-unknown-linux-musl x86_64-linux-musl 
