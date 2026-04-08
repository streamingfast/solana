FROM ubuntu:24.04

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    libssl-dev \
    libudev-dev \
    pkg-config \
    zlib1g-dev \
    llvm \
    clang \
    cmake \
    make \
    libprotobuf-dev \
    protobuf-compiler \
    libclang-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN bash -c "curl https://sh.rustup.rs -sSf | sh -s -- -y"

# Add rustfmt component
RUN bash -c "source $CARGO_HOME/env && rustup component add rustfmt"

# Set working directory
WORKDIR /agave

# Copy the project files
COPY . .

# Build the project in release mode
RUN bash -c "source $CARGO_HOME/env && ./cargo build --release"

# Default command
CMD ["/bin/bash"]
