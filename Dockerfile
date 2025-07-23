# 1. Verwende ein offizielles Rust-Image als Basis
FROM rust:1.87-slim

# 2. Installiere Systemabhängigkeiten
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    curl \
    git \
    bash \
    ca-certificates \
    cmake \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# 3. Setze das Arbeitsverzeichnis
WORKDIR /app

# 4. Kopiere den Projektcode
COPY . .

# 5. Installiere rzup und Toolchain in einem Schritt
# RUN curl -L https://risczero.com/install | bash && \
#     /root/.risc0/bin/rzup install

RUN curl -L https://risczero.com/install | bash && \
    /root/.risc0/bin/rzup install rust 1.85.0 && \
    /root/.risc0/bin/rzup install cpp 2024.1.5 && \
    /root/.risc0/bin/rzup install r0vm 2.3.0 && \
    /root/.risc0/bin/rzup install cargo-risczero 2.3.0 && \
    /root/.risc0/bin/rzup default r0vm 2.3.0

# 6. Setze den PATH dauerhaft
ENV PATH="/root/.risc0/bin:${PATH}"

# 7. Baue dein Projekt
RUN cargo build --release

RUN /root/.risc0/bin/rzup default r0vm 2.1.0
# Server starten
CMD ["target/release/host"]
