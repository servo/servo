# Dockerfile for the servo devcontainer environment.
# Note that the build-context is the repository root.
# We use a multi-stage build to keep the final image size down.

# We use a prebuilt image for `uv` to speed up builds and later copy the artifacts
# into the final stage.
FROM ghcr.io/astral-sh/uv:latest AS uv

FROM ubuntu:24.04 AS base

# Install apt dependencies.
RUN --mount=type=bind,source=python/servo/platform/linux_packages,target=/tmp/linux_packages,readonly \
    apt-get update \
    &&  /tmp/linux_packages/generate_pkg_list.sh /tmp/linux_packages/apt/* | xargs apt-get install -y --no-install-recommends \
    && curl --version

# Please keep `RUST_VERSION` in sync with the `rust-toolchain.toml` file.
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    UV_TOOL_BIN_DIR=/usr/local/bin \
    RUST_VERSION=1.91.0

# Keep the list of components in sync with `rust-toolchain.toml` file.
RUN curl https://sh.rustup.rs -sSf \
        | sh -s -- --default-toolchain ${RUST_VERSION} -y --component clippy,llvm-tools,llvm-tools-preview,rustc-dev,rustfmt,rust-src \
    && \
    rustup --version; \
    cargo --version; \
    rustc --version;

# prebuilt rust tools we use
FROM base AS rust_builder

# TODO: We would need to use `ARG` and install specific versions, to ensure
# that the tools are updated and not always cached.
RUN cargo install cargo-deny cargo-nextest taplo-cli cargo-about --locked


FROM base AS final

COPY --from=rust_builder \
    /usr/local/cargo/bin/cargo-deny \
    /usr/local/cargo/bin/cargo-nextest \
    /usr/local/cargo/bin/taplo \
    /usr/local/cargo/bin/cargo-about \
    /usr/local/cargo/bin/
COPY --from=uv /uv /uvx /bin/