FROM nvidia/cuda:13.0.0-devel-ubuntu24.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    curl \
    git \
    bash \
    ca-certificates \
    cmake \
    protobuf-compiler \
    libclang-dev \
    clang \
    openssh-server \
    openssh-client \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Setup SSH
RUN mkdir -p /run/sshd && \
    mkdir -p /root/.ssh && \
    chmod 700 /root/.ssh

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Setup CUDA paths
ENV PATH="/root/.cargo/bin:/usr/local/cuda/bin:${PATH}"
ENV LD_LIBRARY_PATH="/usr/local/cuda/lib64:${LD_LIBRARY_PATH}"



# Install Risc0
RUN curl -L https://risczero.com/install | bash && \
    /root/.risc0/bin/rzup install rust && \
    /root/.risc0/bin/rzup install cpp && \
    /root/.risc0/bin/rzup install r0vm && \
    /root/.risc0/bin/rzup install cargo-risczero

# Add risc0 to path
ENV PATH="/root/.risc0/bin:${PATH}"

# Create nvcc wrapper to replace -arch=native with sm_89 (L4)
RUN mv /usr/local/cuda/bin/nvcc /usr/local/cuda/bin/nvcc.real && \
    printf '#!/bin/bash\nargs=()\nfor arg in "$@"; do\n  if [ "$arg" = "-arch=native" ]; then\n    args+=("-arch=sm_89")\n  else\n    args+=("$arg")\n  fi\ndone\nexec /usr/local/cuda/bin/nvcc.real "${args[@]}"\n' > /usr/local/cuda/bin/nvcc && \
    chmod +x /usr/local/cuda/bin/nvcc

WORKDIR /app