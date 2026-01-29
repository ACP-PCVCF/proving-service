apt-get update && apt-get install -y \
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
    openssh-client 

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

curl -L https://risczero.com/install | bash && \
    /root/.risc0/bin/rzup install rust && \
    /root/.risc0/bin/rzup install cpp && \
    /root/.risc0/bin/rzup install r0vm && \
    /root/.risc0/bin/rzup install cargo-risczero

PATH="/root/.risc0/bin:${PATH}"

PATH="/root/.cargo/bin:/usr/local/cuda/bin:${PATH}"
LD_LIBRARY_PATH="/usr/local/cuda/lib64:${LD_LIBRARY_PATH}"