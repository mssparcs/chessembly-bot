# Stage 1: Build the application
FROM rust:1.78 AS builder

WORKDIR /usr/src/app

# Copy manifests and build dependencies to cache them
COPY Cargo.toml Cargo.lock ./
# Create a dummy main.rs to build only dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy the actual source code and build the application
COPY src ./src
RUN rm -f target/release/deps/chessembly_bot*
RUN cargo build --release

# Stage 2: Create the final, minimal image
FROM gcr.io/distroless/cc-debian12

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/chessembly-bot /usr/local/bin/chessembly-bot

# Expose the port the app runs on
EXPOSE 8080

# Set the command to run the application
CMD ["chessembly-bot"]
