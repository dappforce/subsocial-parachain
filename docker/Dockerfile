FROM phusion/baseimage:jammy-1.0.1 AS builder

ENV DEBIAN_FRONTEND=noninteractive

WORKDIR /subsocial

COPY . /subsocial

RUN apt-get update && \
    apt-get upgrade -y -o Dpkg::Options::="--force-confold"
    
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
RUN ./scripts/init.sh nosudo

SHELL ["/bin/bash", "-o", "pipefail", "-c"]
RUN . "$HOME/.cargo/env" && cargo build --release

# ==== SECOND STAGE ====

FROM phusion/baseimage:jammy-1.0.1

RUN mv /usr/share/ca* /tmp && \
    rm -rf /usr/share/* && \
    mv /tmp/ca-certificates /usr/share/ && \
    useradd -m -u 1000 -U -s /bin/sh -d /subsocial subsocial

COPY --from=builder /subsocial/target/release/subsocial-collator /usr/local/bin

# checks
RUN ldd /usr/local/bin/subsocial-collator && \
    /usr/local/bin/subsocial-collator --version

# Shrinking
RUN rm -rf /usr/lib/python* && rm -rf /usr/share/man

USER subsocial
RUN mkdir -p /data

VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/subsocial-collator"]