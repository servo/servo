FROM debian:12

ENV RUST_BACKTRACE=1 \
    SHELL=/bin/bash \
    CC=clang \
    CXX=clang++ \
    CARGO_TARGET_DIR=/var/servo-cargo-target \
    UV_PROJECT_ENVIRONMENT=.devcontainer-venv \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo
ENV PATH="${CARGO_HOME}/bin:${PATH}"

WORKDIR /servo-preflight
COPY ci/local-runtime/debian12-preflight.sh ci/local-runtime/debian12-preflight.sh
COPY rust-toolchain.toml rust-toolchain.toml
RUN chmod +x ci/local-runtime/debian12-preflight.sh \
    && ci/local-runtime/debian12-preflight.sh \
    && mkdir -p "${CARGO_TARGET_DIR}" \
    && chmod 0777 "${CARGO_TARGET_DIR}" \
    && rm rust-toolchain.toml
