# syntax=docker/dockerfile:1
#
# lume build environment: JDK, Android SDK/NDK, Rust and cargo-ndk at pinned versions.
# Uso: scripts/container-build.sh [debug|release]
#
# Stages: toolchain -> build -> artifacts (FROM scratch, only the APK/AAB files).

FROM debian:bookworm-slim AS toolchain

ARG ANDROID_CMDLINE_TOOLS=15859902
ARG ANDROID_PLATFORM=36
ARG ANDROID_BUILD_TOOLS=36.0.0
ARG ANDROID_NDK=28.2.13676358
ARG CARGO_NDK_VERSION=4.1.2

ENV DEBIAN_FRONTEND=noninteractive \
    ANDROID_HOME=/opt/android-sdk \
    ANDROID_SDK_ROOT=/opt/android-sdk \
    ANDROID_NDK_HOME=/opt/android-sdk/ndk/${ANDROID_NDK} \
    RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:/opt/android-sdk/cmdline-tools/latest/bin:/opt/android-sdk/build-tools/${ANDROID_BUILD_TOOLS}:$PATH

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates curl unzip build-essential pkg-config openjdk-17-jdk-headless \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL -o /tmp/cmdline-tools.zip \
        https://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_CMDLINE_TOOLS}_latest.zip \
    && mkdir -p ${ANDROID_HOME}/cmdline-tools \
    && unzip -q /tmp/cmdline-tools.zip -d ${ANDROID_HOME}/cmdline-tools \
    && mv ${ANDROID_HOME}/cmdline-tools/cmdline-tools ${ANDROID_HOME}/cmdline-tools/latest \
    && rm /tmp/cmdline-tools.zip \
    && yes | sdkmanager --licenses >/dev/null \
    && sdkmanager "platforms;android-${ANDROID_PLATFORM}" "build-tools;${ANDROID_BUILD_TOOLS}" "ndk;${ANDROID_NDK}"

# Rust at the version in rust-toolchain.toml (with the Android targets) and a pinned cargo-ndk.
COPY rust-toolchain.toml /tmp/rust-toolchain.toml
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none \
    && cd /tmp && rustup toolchain install \
    && cargo install cargo-ndk --version ${CARGO_NDK_VERSION} --locked \
    && rm -rf /opt/cargo/registry

# The Android NDK's Linux host tools are x86_64 even when the builder runs on ARM64.
RUN if [ "$(dpkg --print-architecture)" = arm64 ]; then \
      dpkg --add-architecture amd64; \
      apt-get update; \
      apt-get install -y --no-install-recommends libc6:amd64 libstdc++6:amd64 zlib1g:amd64; \
      rm -rf /var/lib/apt/lists/*; \
    fi

FROM toolchain AS build

ARG VARIANT=debug
# CPU/memory limits so the build fits in WSL (fat LTO is heavy).
ARG LUME_VERSION_NAME=
ARG LUME_VERSION_CODE=
ARG CARGO_BUILD_JOBS=4
ARG GRADLE_HEAP=1536m
ENV LUME_VERSION_NAME=${LUME_VERSION_NAME} \
    LUME_VERSION_CODE=${LUME_VERSION_CODE} \
    CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS} \
    GRADLE_OPTS="-Xmx${GRADLE_HEAP} -Dorg.gradle.daemon=false"

WORKDIR /src
COPY . /src

# Optional release signing, passed as build secrets (they never end up in an image layer):
# lume_keystore (the keystore file), lume_store_password, lume_key_alias and lume_key_password.
RUN --mount=type=cache,target=/opt/cargo/registry \
    --mount=type=cache,target=/opt/cargo/git \
    --mount=type=cache,target=/src/target \
    --mount=type=cache,target=/root/.gradle \
    --mount=type=secret,id=lume_keystore,required=false \
    --mount=type=secret,id=lume_store_password,required=false \
    --mount=type=secret,id=lume_key_alias,required=false \
    --mount=type=secret,id=lume_key_password,required=false \
    --mount=type=secret,id=lume_keys_file,required=false \
    set -e; \
    if [ -f /run/secrets/lume_keystore ]; then \
        export LUME_KEYSTORE=/run/secrets/lume_keystore; \
        export LUME_KEYSTORE_PASSWORD="$(cat /run/secrets/lume_store_password)"; \
        export LUME_KEY_ALIAS="$(cat /run/secrets/lume_key_alias)"; \
        export LUME_KEY_PASSWORD="$(cat /run/secrets/lume_key_password)"; \
    fi; \
    if [ -f /run/secrets/lume_keys_file ]; then \
        export LUME_KEYS_FILE=/run/secrets/lume_keys_file; \
    fi; \
    # The project's sources must look newer than anything in the cached target directory: with a
    # container clock ahead of the host's, cargo took fresh edits for already compiled code.
    find /src/crates /src/tools -name '*.rs' -exec touch {} +; \
    cd android; \
    if [ "${VARIANT}" = "release" ]; then \
        ./gradlew --no-daemon assembleRelease bundleRelease; \
    else \
        ./gradlew --no-daemon assembleDebug; \
    fi; \
    mkdir -p /out; \
    find app/build/outputs -name '*.apk' -o -name '*.aab' | while read f; do cp "$f" /out/; done; \
    ls -l /out

FROM scratch AS artifacts
COPY --from=build /out/ /
