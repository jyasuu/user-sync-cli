FROM rust:1.94.0 AS builder

# Set the working directory
WORKDIR /usr/src/app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY ./src ./src

# Build the project in release mode
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine

# Copy the compiled binary from the build stage
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/user-sync /usr/local/bin/user-sync

# Set the startup command

CMD ["user-sync"]
