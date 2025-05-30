# Use official Rust image as build environment
FROM rust:1.82 as builder


# Create app directory
WORKDIR /usr/src/app

# Copy manifests first (for caching dependencies)
COPY host ./host
COPY Cargo.toml ./Cargo.toml 
COPY Cargo.lock ./Cargo.lock 
COPY target ./target
COPY methods ./methods
COPY host/src ./src

RUN rustc --version

# Build in release mode
RUN cargo build 


# Use a smaller image for runtime
FROM debian:buster-slim

# Install required dependencies for Kafka and your app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder stage
COPY --from=builder /usr/src/app/target/release/your_binary_name /usr/local/bin/your_binary_name

# Set workdir
WORKDIR /usr/local/bin

# Run the app
CMD ["./host"]
