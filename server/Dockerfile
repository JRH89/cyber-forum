# Use the official Rust image
FROM rust:1.70 as builder

# Create a new empty shell project
WORKDIR /usr/src/forum-server

# Copy the manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

# Cache dependencies
RUN cargo build --release

# Now copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Use a minimal runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl1.1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder
COPY --from=builder /usr/src/forum-server/target/release/forum-server /usr/local/bin/

# Set the working directory
WORKDIR /usr/local/bin

# Set environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

# Expose the port the app runs on
EXPOSE 8080

# Run the application
CMD ["forum-server"]
