FROM rust:1.88-alpine AS builder
RUN apk add --no-cache musl-dev pkgconf openssl-dev openssl-libs-static

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY services/openhack-common/Cargo.toml services/openhack-common/Cargo.toml
COPY services/auth-rust/Cargo.toml services/auth-rust/Cargo.toml
COPY services/core-rust/Cargo.toml services/core-rust/Cargo.toml
COPY services/gateway-rust/Cargo.toml services/gateway-rust/Cargo.toml
COPY services/judging-rust/Cargo.toml services/judging-rust/Cargo.toml
COPY services/leaderboard/Cargo.toml services/leaderboard/Cargo.toml
COPY services/ai-rust/Cargo.toml services/ai-rust/Cargo.toml
COPY services/analytics-rust/Cargo.toml services/analytics-rust/Cargo.toml
COPY services/mail-rust/Cargo.toml services/mail-rust/Cargo.toml
COPY services/media-rust/Cargo.toml services/media-rust/Cargo.toml
COPY services/notify-rust/Cargo.toml services/notify-rust/Cargo.toml
COPY services/sponsors-rust/Cargo.toml services/sponsors-rust/Cargo.toml
COPY services/discord-bot-rust/Cargo.toml services/discord-bot-rust/Cargo.toml
COPY cli-rust/Cargo.toml cli-rust/Cargo.toml

RUN mkdir -p services/openhack-common/src && touch services/openhack-common/src/lib.rs && \
    for dir in services/auth-rust services/core-rust services/gateway-rust \
      services/judging-rust services/leaderboard services/ai-rust \
      services/analytics-rust services/mail-rust services/media-rust \
      services/notify-rust services/sponsors-rust services/discord-bot-rust \
      cli-rust; do \
      mkdir -p "$dir/src" && echo "fn main() {}" > "$dir/src/main.rs"; \
    done
RUN cargo build --release -p media-service 2>/dev/null || true

COPY services/openhack-common/src/ services/openhack-common/src/
COPY services/media-rust/src/ services/media-rust/src/
RUN touch services/openhack-common/src/lib.rs services/media-rust/src/main.rs && cargo build --release -p media-service

FROM alpine:3.21 AS container
RUN apk add --no-cache ca-certificates curl
COPY --from=builder /app/target/release/media-service /usr/local/bin/
EXPOSE 3010
CMD ["media-service"]

FROM alpine:3.21 AS lambda
RUN apk add --no-cache ca-certificates
COPY --from=public.ecr.aws/awsguru/aws-lambda-web-adapter:0.8.4 /lambda-adapter /opt/extensions/lambda-adapter
COPY --from=builder /app/target/release/media-service /usr/local/bin/
ENV PORT=8080 RUST_LOG=info
CMD ["/usr/local/bin/media-service"]
