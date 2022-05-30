FROM rust:latest as builder

# Make a fake Rust app to keep a cached layer of compiled crates
RUN USER=root cargo new app
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
# Needs at least a main.rs file with a main function
RUN mkdir src && echo "fn main(){}" > src/main.rs
# Will build all dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release

# Copy the rest
COPY . .
# Build (install) the actual binaries
RUN cargo install --path .

# Runtime image
FROM debian:bullseye

# https://github.com/awslabs/aws-sdk-rust/discussions/434#discussioncomment-2090346
# thread 'main' panicked at 'no CA certificates found',
# /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/hyper-rustls-0.22.1/src/connector.rs:45:13
RUN apt-get update
RUN apt-get install -y ca-certificates
RUN update-ca-certificates

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/local/cargo/bin/hello /app/hello
COPY mikoto.json /app/mikoto.json

# No CMD or ENTRYPOINT, see fly.toml with `cmd` override.
