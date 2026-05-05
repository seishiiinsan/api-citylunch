# Stage 1 : Build
FROM rust:1.76 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Stage 2 : Image finale
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api-citylunch /usr/local/bin/api-citylunch
COPY migrations ./migrations
EXPOSE 3000
CMD ["api-citylunch"]
