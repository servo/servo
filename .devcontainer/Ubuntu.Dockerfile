# Dockerfile for the servo devcontainer environment.
# Note that the build-context is the repository root.
# We use a multi-stage build to keep the final image size down.

# We use a prebuilt image for `uv` to speed up builds and later copy the artifacts
# into the final stage.
FROM ghcr.io/astral-sh/uv:latest AS uv

FROM ubuntu:24.04 AS base

# Install apt dependencies.
COPY python/servo/platform/linux_packages /tmp/linux_packages
RUN apt-get update \
    &&  /tmp/linux_packages/generate_pkg_list.sh /tmp/linux_packages/apt/* | xargs apt-get install -y --no-install-recommends \
    && curl --version

# Required due to https://github.com/servo/servo/issues/35029
RUN apt purge -y fonts-droid-fallback

# Please keep `RUST_VERSION` in sync with the `rust-toolchain.toml` file.
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    UV_TOOL_BIN_DIR=/usr/local/bin \
    RUST_VERSION=1.95.0

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
RUN cargo install cargo-deny cargo-nextest taplo-cli --locked


FROM base AS final

COPY --from=rust_builder \
    /usr/local/cargo/bin/cargo-deny \
    /usr/local/cargo/bin/cargo-nextest \
    /usr/local/cargo/bin/taplo \
    /usr/local/cargo/bin/
COPY --from=uv /uv /uvx /bin/


# Image with Android SDK
FROM final AS android

# Keep versions in sync with build.gradle.kts
ARG JAVA_VERSION=21
ARG ANDROID_SDK_VERSION=34
ARG ANDROID_BUILD_TOOLS_VERSION=34.0.0
ARG ANDROID_NDK_VERSION=28.2.13676358
ENV ANDROID_SDK_ROOT=/opt/android-sdk
ENV ANDROID_NDK_ROOT=${ANDROID_SDK_ROOT}/ndk/${ANDROID_NDK_VERSION}/
RUN apt-get update \
    && apt-get install -y --no-install-recommends openjdk-${JAVA_VERSION}-jdk unzip \
    && android_cmdline_tools_url="$(curl https://developer.android.com/studio | grep -o "https:\/\/dl.google.com\/android\/repository\/commandlinetools\-linux\-[0-9]*_latest\.zip")" \
    && curl "${android_cmdline_tools_url}" -sSLf -o /tmp/android-commandlinetools-linux.zip \
    && unzip -q /tmp/android-commandlinetools-linux.zip -d ${ANDROID_SDK_ROOT} \
    && rm /tmp/android-commandlinetools-linux.zip \
    && mv ${ANDROID_SDK_ROOT}/cmdline-tools ${ANDROID_SDK_ROOT}/latest && mkdir -p ${ANDROID_SDK_ROOT}/cmdline-tools && mv ${ANDROID_SDK_ROOT}/latest ${ANDROID_SDK_ROOT}/cmdline-tools/
RUN yes | ${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin/sdkmanager --licenses \
    && ${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin/sdkmanager --install \
    "build-tools;${ANDROID_BUILD_TOOLS_VERSION}" \
    "ndk;${ANDROID_NDK_VERSION}" \
    "platform-tools" \
    "platforms;android-${ANDROID_SDK_VERSION}"
