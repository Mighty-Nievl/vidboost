# Backend Rust + ffmpeg untuk VidBoost.
# Frontend tidak ikut ke image ini — frontend di-deploy ke Cloudflare Pages.
FROM rust:1-bookworm AS builder
WORKDIR /app
COPY server-rs/Cargo.toml server-rs/Cargo.lock ./
COPY server-rs/src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
      ffmpeg \
      fonts-dejavu-core \
      ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/vidboost-server /usr/local/bin/vidboost-server

ENV PORT=8080 \
    DATA_DIR=/data \
    FFMPEG_PATH=ffmpeg \
    FFPROBE_PATH=ffprobe \
    DRAWTEXT_FONTFILE=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf

RUN mkdir -p /data
VOLUME ["/data"]
EXPOSE 8080

CMD ["vidboost-server"]
