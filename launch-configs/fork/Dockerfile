FROM node:18-bookworm

RUN apt-get update && apt-get install -y \
    curl \
    wget \
    nodejs \
    npm \
    ca-certificates \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /zombienet/Basilisk-node/launch-configs/fork

RUN wget -q https://github.com/paritytech/zombienet/releases/download/v1.3.128/zombienet-linux-x64 -O /usr/local/bin/zombienet && \
    chmod +x /usr/local/bin/zombienet
RUN zombienet version

ARG POLKADOT_SDK_VERSION=latest

RUN mkdir -p /zombienet/polkadot-sdk/target/release
RUN if [ "$POLKADOT_SDK_VERSION" = "latest" ]; then \
        DOWNLOAD_URL="https://github.com/paritytech/polkadot-sdk/releases/latest/download"; \
    else \
        DOWNLOAD_URL="https://github.com/paritytech/polkadot-sdk/releases/download/$POLKADOT_SDK_VERSION"; \
    fi && \
    for binary in polkadot polkadot-execute-worker polkadot-prepare-worker; do \
        wget -q $DOWNLOAD_URL/$binary \
            -O /zombienet/polkadot-sdk/target/release/$binary \
        && chmod +x /zombienet/polkadot-sdk/target/release/$binary \
        && /zombienet/polkadot-sdk/target/release/$binary --version; \
    done

ARG BASILISK_BIN_URL=https://github.com/galacticcouncil/Basilisk-node/releases/latest/download
ARG SCRAPER_URL=https://github.com/galacticcouncil/hydration-node/releases/latest/download

RUN mkdir -p /zombienet/Basilisk-node/target/release
RUN wget -q $BASILISK_BIN_URL/basilisk -O /zombienet/Basilisk-node/target/release/basilisk && \
    wget -q $SCRAPER_URL/scraper -O /zombienet/Basilisk-node/target/release/scraper && \
    chmod +x /zombienet/Basilisk-node/target/release/*
RUN /zombienet/Basilisk-node/target/release/basilisk --version

COPY . .

RUN npm i

VOLUME /zombienet/Basilisk-node/launch-configs/fork/data
EXPOSE 9988

CMD npm start
