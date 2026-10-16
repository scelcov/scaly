FROM rust:1.99-bookworm AS build

WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
ENV CARGO_PROFILE_RELEASE_LTO=thin
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=8
RUN cargo build --release --locked -p upscale

FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        ffmpeg \
        libvulkan1 \
        mesa-vulkan-drivers \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /src/target/release/upscale /usr/local/bin/upscale

EXPOSE 8788
ENTRYPOINT ["upscale"]
CMD ["mcp", "--addr", "0.0.0.0:8788"]
