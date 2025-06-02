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
RUN curl -L https://risczero.com/install | bash && \
    /root/.risc0/bin/rzup install

# 6. Setze den PATH dauerhaft
ENV PATH="/root/.risc0/bin:${PATH}"

# 7. Baue dein Projekt
RUN cargo build --release

# Port für den Webserver
EXPOSE 3000

# Server starten
CMD ["target/release/host"]