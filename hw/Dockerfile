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
# The build runs the full hardware gate:
#   make -C hw check                       (sim, golden, gatesim, equiv, synth)
#   bash hw/synth/yosys/run_pnr.sh         (PnR, bitstream, icetime report)

FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# Packaged toolchain: Verilator, Yosys, nextpnr-ice40, Python, build tools.
RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    curl \
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

# Verify the toolchain. nextpnr-ice40, icepack, and icetime in this
# image do not support --version, so their presence is checked with
# command -v; only verilator and yosys print a version.
RUN verilator --version && yosys --version \
    && command -v nextpnr-ice40 && command -v icepack && command -v icetime

WORKDIR /workspace

# Build-time: run the hardware verification gate.
COPY . .
RUN make -C hw check
RUN bash hw/synth/yosys/run_pnr.sh

ENTRYPOINT ["bash"]
