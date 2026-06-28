# Use official Ubuntu 24.04 LTS as the base image
FROM ubuntu:24.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies including GCC 14, LLVM 18, Clang 18, CMake, and RocksDB build essentials
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    gcc-14 \
    g++-14 \
    clang-18 \
    llvm-18 \
    libclang-18-dev \
    cmake \
    librocksdb-dev \
    libbz2-dev \
    zlib1g-dev \
    libsnappy-dev \
    liblz4-dev \
    libzstd-dev \
    sudo \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set GCC 14 and Clang 18 as the default compilers
RUN update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-14 100 \
    && update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-14 100 \
    && update-alternatives --install /usr/bin/clang clang /usr/bin/clang-18 100 \
    && update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-18 100

# Set environment variables for RocksDB / LLVM building
ENV CC=gcc
ENV CXX=g++
ENV CLANG_PATH=/usr/bin/clang
ENV LIBCLANG_PATH=/usr/lib/llvm-18/lib

# Install Rust toolchain 1.96.0 (using rustup)
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.96.0 --profile default

# Verify tool versions
RUN rustc --version && cargo --version && gcc --version && cmake --version && clang --version
