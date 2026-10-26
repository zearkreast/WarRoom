# syntax=docker/dockerfile:1
# WarRoom collector. Build from the repo root:  docker compose up -d --build
FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY collector collector
COPY vendor vendor
# The workspace's root package is the app, which the collector doesn't need; a stub lets Cargo read the workspace.
RUN mkdir src && echo 'fn main() {}' > src/main.rs
# The caches keep downloaded crates and compiled dependencies between builds, so an update rebuilds only what changed.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --locked --profile collector -p warroom-collector \
    && cp target/collector/warroom-collector /usr/local/bin/

FROM debian:bookworm-slim
COPY --from=build /usr/local/bin/warroom-collector /usr/local/bin/
USER nobody
CMD ["warroom-collector"]
