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
#   bash hw/synth/yosys/run_pnr.sh   (PnR, bitstream, icetime report)
#   make -C hw coverage              (3 channels x 11,172 + formal equivalence)

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

# icestorm (icepack, icetime, chipdb) is not packaged in Ubuntu 24.04;
# build from source. Only the subdirectories the flow needs are built
# (icebox chipdb, icepack, icetime); iceprog is skipped because it needs
# libftdi1-dev. Default prefix /usr/local puts the chipdb under
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

# Verify the toolchain. nextpnr-ice40, icepack, and icetime in this
# image do not support --version, so their presence is checked with
# command -v; only verilator and yosys print a version.
RUN verilator --version && yosys --version \
    && command -v nextpnr-ice40 && command -v icepack && command -v icetime

WORKDIR /workspace

# Build-time: run the full project verification (fmt, clippy, build, test,
# C++ core, and the hardware gate via check_hw), the demo PnR flow, and
# the explicit coverage summary.
COPY . .
RUN bash run.sh --check
RUN bash hw/synth/yosys/run_pnr.sh
RUN make -C hw coverage

ENTRYPOINT ["bash"]
