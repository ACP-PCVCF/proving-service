FROM nvidia/cuda:13.0.0-devel-ubuntu24.04

# Copy environment_install.sh script to the container and execute it to set up the environment
#COPY environment_install.sh /tmp/environment_install.sh
#RUN chmod +x /tmp/environment_install.sh && /tmp/environment_install.sh

ENV DEBIAN_FRONTEND=noninteractive
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

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:/usr/local/cuda/bin:${PATH}"
ENV LD_LIBRARY_PATH="/usr/lib/x86_64-linux-gnu:/usr/local/cuda/lib64:${LD_LIBRARY_PATH}"
RUN curl -L https://risczero.com/install | bash && \
    /root/.risc0/bin/rzup install rust && \
    /root/.risc0/bin/rzup install cpp && \
    /root/.risc0/bin/rzup install r0vm && \
    /root/.risc0/bin/rzup install cargo-risczero

ENV PATH="/root/.risc0/bin:${PATH}"