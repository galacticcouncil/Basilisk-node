FROM ubuntu:21.04

WORKDIR /basilisk

RUN useradd -m -u 1000 -U -s /bin/sh -d /basilisk basilisk && \
    mkdir -p /basilisk/.local/share/basilisk && \
    ln -s /basilisk/.local/share/basilisk /data && \
    chown -R basilisk:basilisk /basilisk

ADD ./basilisk /basilisk/basilisk
RUN ln -f /basilisk/basilisk /basilisk/testing-basilisk

RUN chown basilisk: -R /basilisk

USER basilisk

EXPOSE 30333 9933 9944

VOLUME ["/data"]

ENTRYPOINT [ "/basilisk/basilisk" ]
