# syntax=docker/dockerfile:1
# SPDX-License-Identifier: CERN-OHL-S-2.0
# Copyright (C) 2026 Taeho Lee
#
# This source describes Open Hardware and is licensed under the CERN-OHL-P v2.
# You may redistribute and modify this documentation and make products using
# it under the terms of the CERN-OHL-P v2 (https://cern.ch/cern-ohl). This
# documentation is distributed WITHOUT ANY EXPRESS OR IMPLIED WARRANTY,
# INCLUDING OF MERCHANTABILITY, SATISFACTORY QUALITY AND FITNESS FOR A
# PARTICULAR PURPOSE. Please see the CERN-OHL-P v2 for applicable conditions.
#
# Hardware verification environment for the Tagma decoder.
#
# Packages the open hardware toolchain so the verification gate runs
# reproducibly in CI. Follows the ev/Dockerfile pattern: base image,
# toolchain install, source builds for un-packaged tools, then the gate
# as build-time smoke tests.
#
# Build (context must be the repository root):
#   docker build -f hw/Dockerfile -t tagma-hw .
#
# The build runs the full project verification as build-time smoke tests:
#   bash run.sh --check              (fmt, clippy, build, test, C++ core, hw gate)
#   make -C hw coverage              (every sim channel, PnR, and formal equivalence)

FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# Packaged toolchain: Verilator, Yosys, nextpnr-ice40, Python, build tools.
# cmake is required by the C++ core in sw/cpp, which run.sh --check builds.
RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    curl \
    cmake \
    python3 \
    yosys \
    verilator \
    nextpnr-ice40 \
    && rm -rf /var/lib/apt/lists/*

# icestorm (icepack, icetime, icebox_vlog, chipdb) is not packaged in Ubuntu
# 24.04; build from source. Only the subdirectories the flow needs are built
# (icebox for the chipdb and icebox_vlog, icepack, icetime); iceprog is skipped
# because it needs libftdi1-dev. Default prefix /usr/local puts the chipdb under
# share/icebox, which icetime resolves without extra setup on Linux.
RUN git clone https://github.com/YosysHQ/icestorm.git /tmp/icestorm \
    && cd /tmp/icestorm \
    && make -C icebox && make -C icepack && make -C icetime \
    && make -C icebox install && make -C icepack install && make -C icetime install \
    && rm -rf /tmp/icestorm

# Rust for the golden-export reference exporter and the benchmark center.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# The MCU target for the no_std storage path: run.sh --check compiles
# tagma-core/geo/map for riscv32imac-unknown-none-elf.
RUN rustup target add riscv32imac-unknown-none-elf

# sccache, the compile cache. A layer cache cannot reach past `COPY . .`: one
# source edit changes every layer above it, so the Rust and C++ objects are
# rebuilt from scratch. A compiler cache is keyed on the translation unit, and
# the GitHub Actions backend puts the objects in the same cache service the
# layers use, so a fresh runner still hits them.
ARG TARGETARCH
ARG SCCACHE_VERSION=0.10.0
# TARGETARCH is set by buildx; a plain `docker build` leaves it empty, so the
# mapping falls back to the container's own machine name.
RUN case "${TARGETARCH:-$(uname -m)}" in \
      amd64|x86_64) ARCH=x86_64 ;; \
      arm64|aarch64) ARCH=aarch64 ;; \
      *) echo "unsupported TARGETARCH: ${TARGETARCH:-$(uname -m)}"; exit 1 ;; \
    esac \
    && ASSET="sccache-v${SCCACHE_VERSION}-${ARCH}-unknown-linux-musl" \
    && curl -fsSL -o /tmp/${ASSET}.tar.gz \
         "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/${ASSET}.tar.gz" \
    && tar -xzf /tmp/${ASSET}.tar.gz -C /tmp \
    && install -m 0755 /tmp/${ASSET}/sccache /usr/local/bin/sccache \
    && rm -rf /tmp/${ASSET}* \
    && sccache --version

# Verify the toolchain. nextpnr-ice40, icepack, icetime, and icebox_vlog
# in this image do not support --version, so their presence is checked with
# command -v; only verilator and yosys print a version.
RUN verilator --version && yosys --version \
    && command -v nextpnr-ice40 && command -v icepack && command -v icetime \
    && command -v icebox_vlog

WORKDIR /workspace

# Build-time: run the full project verification (fmt, clippy, build, test,
# C++ core, and the hardware gate via check_hw, which now covers the demo PnR
# and the recovered-bitstream channel), then the coverage summary. The summary
# is a separate target because the gate above already ran every channel: the
# former `make -C hw coverage` depended on the same channels and repeated them
# for the four lines it prints.
#
# The compile cache is enabled only when the runner hands its coordinates in
# and is inert otherwise: without them sccache keeps its objects in the
# container and the gate behaves exactly as before. Both arrive as build
# secrets rather than build args, because a build arg is part of the layer cache
# key and the URL changes per job, which would invalidate this layer on every
# run. RUSTC_WRAPPER covers the Rust workspace and CMAKE_CXX_COMPILER_LAUNCHER
# covers the C++ core, which CMake reads from the environment as the initial
# cache value.
COPY . .
RUN --mount=type=secret,id=actions_results_url,env=ACTIONS_RESULTS_URL,required=false \
    --mount=type=secret,id=actions_runtime_token,env=ACTIONS_RUNTIME_TOKEN,required=false \
    bash -c 'set -e; \
      export RUSTC_WRAPPER=sccache CMAKE_CXX_COMPILER_LAUNCHER=sccache; \
      if [ -n "${ACTIONS_RUNTIME_TOKEN:-}" ] && [ -n "${ACTIONS_RESULTS_URL:-}" ]; then \
        export SCCACHE_GHA_ENABLED=on; \
      fi; \
      bash run.sh --check; \
      sccache --show-stats || true'
RUN make -C hw coverage-summary

ENTRYPOINT ["bash"]
