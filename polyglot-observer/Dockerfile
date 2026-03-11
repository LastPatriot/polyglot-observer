# Use a multi-stage build to keep the final image small
FROM rust:1.83-slim AS builder

WORKDIR /usr/src/app
COPY . .

# Install dependencies for building
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Build the application
RUN cargo build --release

# Final runtime image
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder
COPY --from=builder /usr/src/app/target/release/polyglot-observer /usr/local/bin/polyglot-observer

# The app expects config.toml in the working directory
# We'll mount it via ConfigMap in K8s, but let's set the entrypoint
CMD ["polyglot-observer"]
