FROM ubuntu:21.04

RUN apt update && apt install -y ca-certificates

workdir /basilisk

RUN useradd -m -u 1000 -U -s /bin/sh -d /basilisk basilisk && \
    mkdir -p /basilisk/.local/share/basilisk && \
    ln -s /basilisk/.local/share/basilisk /data && \
    chown -R basilisk:basilisk /basilisk
    
USER basilisk
ADD ./basilisk /basilisk/basilisk

EXPOSE 30333 9933 9944

VOLUME ["/data"]

entrypoint [ "/basilisk/basilisk" ]
