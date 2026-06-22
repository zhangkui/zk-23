FROM rust:1.75-alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

COPY Cargo.toml Cargo.lock ./
COPY .cargo ./.cargo

RUN mkdir -p src && echo "fn main() {}" > src/main.rs

RUN cargo build --release --locked 2>/dev/null || true

COPY . .

RUN cargo build --release --locked

FROM alpine:3.19

WORKDIR /app

RUN apk add --no-cache ca-certificates openssl tzdata

ENV TZ=Asia/Shanghai

COPY --from=builder /app/target/release/cableway-monitor /app/cableway-monitor
COPY --from=builder /app/.env.example /app/.env

RUN chmod +x /app/cableway-monitor

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8080/api/health || exit 1

ENTRYPOINT ["/app/cableway-monitor"]

CMD ["--migrate", "--init-data"]
