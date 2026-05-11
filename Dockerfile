# Build on Debian trixie (glibc aligns with distroless cc-debian13)
FROM rust:1-trixie AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release

# Runtime: minimal libc/C++ runtime + CA bundle; :nonroot runs as uid/gid 65532
FROM gcr.io/distroless/cc-debian13:nonroot

WORKDIR /app

# App loads File::with_name("config") from cwd → needs config.toml here (env APP__* overrides)
COPY --from=builder /app/config.toml /app/config.toml

COPY --from=builder /app/target/release/messages-hook /usr/local/bin/messages-hook

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/messages-hook"]
