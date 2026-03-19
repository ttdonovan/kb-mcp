# kb-mcp Researcher Agent — Multi-Stage Build
# Builds ZeroClaw, kb-mcp, and Earl from source into a minimal runtime image.
#
# Build: just agent-build
# Run:   just agent-research

# ---------------------------------------------------------------------------
# Stage 1: Build ZeroClaw from source
# ---------------------------------------------------------------------------
FROM rust:slim-bookworm AS build-zeroclaw

RUN apt-get update && apt-get install -y --no-install-recommends \
    git pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN git clone --depth 1 https://github.com/zeroclaw-labs/zeroclaw.git /tmp/zc \
    && cd /tmp/zc \
    && cargo build --release \
    && cp target/release/zeroclaw /usr/local/bin/zeroclaw \
    && rm -rf /tmp/zc

# ---------------------------------------------------------------------------
# Stage 2: Build kb-mcp from source
# ---------------------------------------------------------------------------
FROM rust:slim-bookworm AS build-kbmcp

RUN apt-get update && apt-get install -y --no-install-recommends \
    git pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN git clone --depth 1 https://github.com/ttdonovan/kb-mcp.git /tmp/kb \
    && cd /tmp/kb \
    && cargo build --release \
    && cp target/release/kb-mcp /usr/local/bin/kb-mcp \
    && rm -rf /tmp/kb

# ---------------------------------------------------------------------------
# Stage 3: Build Earl from source (minimal: http only)
# Earl's build.rs requires Node.js + pnpm for web assets even with
# minimal features — install them in the builder stage.
# ---------------------------------------------------------------------------
FROM rust:slim-bookworm AS build-earl

RUN apt-get update && apt-get install -y --no-install-recommends \
    git pkg-config libssl-dev ca-certificates make perl curl \
    && rm -rf /var/lib/apt/lists/*

# Node.js + pnpm (required by Earl's build.rs)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && npm install -g pnpm@latest \
    && rm -rf /var/lib/apt/lists/*

RUN git clone --depth 1 https://github.com/brwse/earl.git /tmp/earl \
    && cd /tmp/earl \
    && cargo build --release --no-default-features --features http \
    && cp target/release/earl /usr/local/bin/earl \
    && rm -rf /tmp/earl

# ---------------------------------------------------------------------------
# Stage 4: Runtime
# ---------------------------------------------------------------------------
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl jq \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries from build stages
COPY --from=build-zeroclaw /usr/local/bin/zeroclaw /usr/local/bin/zeroclaw
COPY --from=build-kbmcp /usr/local/bin/kb-mcp /usr/local/bin/kb-mcp
COPY --from=build-earl /usr/local/bin/earl /usr/local/bin/earl

# Shared entrypoint — copies config into ZeroClaw's tmpfs config dir
COPY scripts/entrypoint.sh /usr/local/bin/entrypoint.sh

# Non-root agent user
RUN groupadd -g 1001 agent && \
    useradd -u 1001 -g agent -m -s /bin/bash agent

USER agent
WORKDIR /workspace

ENTRYPOINT ["entrypoint.sh"]
CMD ["zeroclaw", "agent"]
