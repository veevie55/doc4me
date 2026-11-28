FROM rust:1-bookworm

USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential cmake libclang-dev libasound2-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*
