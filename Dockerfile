FROM debian:bullseye-slim

# https://github.com/awslabs/aws-sdk-rust/discussions/434#discussioncomment-2090346
# thread 'main' panicked at 'no CA certificates found',
# /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/hyper-rustls-0.22.1/src/connector.rs:45:13
RUN apt-get update
RUN apt-get install -y ca-certificates
RUN update-ca-certificates

WORKDIR /app

COPY target/release/hello /app/hello
COPY mikoto.json /app/mikoto.json
RUN ls -alh /app
RUN chmod +x /app/*
