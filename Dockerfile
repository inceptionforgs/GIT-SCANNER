FROM rust:latest as builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/git-scanner .

RUN apt-get update && apt-get install -y ca-certificates

CMD ["./git-scanner"]