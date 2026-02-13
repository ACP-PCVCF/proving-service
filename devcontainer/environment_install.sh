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
    openssh-client \
    libcurl4-openssl-dev \
    cuda

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

curl -L https://risczero.com/install | bash && \
    /root/.risc0/bin/rzup install rust && \
    /root/.risc0/bin/rzup install cpp && \
    /root/.risc0/bin/rzup install r0vm && \
    /root/.risc0/bin/rzup install cargo-risczero

PATH="/root/.risc0/bin:${PATH}"

PATH="/root/.cargo/bin:/usr/local/cuda/bin:${PATH}"
LD_LIBRARY_PATH="/usr/lib/x86_64-linux-gnu:/usr/local/cuda/lib64:${LD_LIBRARY_PATH}"
#LD_LIBRARY_PATH="/usr/local/cuda/lib64:${LD_LIBRARY_PATH}"

mv /usr/local/cuda/bin/nvcc /usr/local/cuda/bin/nvcc.real && \
    printf '#!/bin/bash\nargs=()\nfor arg in "$@"; do\n  if [ "$arg" = "-arch=native" ]; then\n    args+=("-arch=sm_89")\n  else\n    args+=("$arg")\n  fi\ndone\nexec /usr/local/cuda/bin/nvcc.real "${args[@]}"\n' > /usr/local/cuda/bin/nvcc && \
    chmod +x /usr/local/cuda/bin/nvcc