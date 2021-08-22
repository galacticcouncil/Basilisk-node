# -- finale image
FROM ubuntu:20.04

workdir /polkadot

RUN useradd -m -u 1000 -U -s /bin/sh -d /polkadot polkadot && \
    mkdir -p /polkadot/.local/share/polkadot && \
    ln -s /polkadot/.local/share/polkadot /data && \
    chown -R polkadot:polkadot /polkadot
    
USER polkadot

EXPOSE 30333 9933 9944

VOLUME ["/data"]
cmd [ "/bin/bash" ]
