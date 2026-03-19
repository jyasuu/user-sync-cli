FROM rust:1.94.0 AS builder

# Set the working directory
WORKDIR /usr/src/app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY ./src ./src

# 1. Install the C toolchain for musl
RUN apt-get update && apt-get install -y \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# 2. Add the rust target
RUN rustup target add x86_64-unknown-linux-musl

# 3. Set the environment variable so 'cc-rs' knows which compiler to use
ENV CC_x86_64_unknown_linux_musl=musl-gcc

RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine

# Copy the compiled binary from the build stage
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/user-sync /usr/local/bin/user-sync

# Set the startup command

CMD ["user-sync"]
