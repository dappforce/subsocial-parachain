# Based on https://github.com/AcalaNetwork/Acala/Dockerfile

FROM phusion/baseimage:0.10.2 as builder
LABEL maintainer="dappforce@pm.me"
LABEL description="This is the build stage for Subsocial PC Node. Here we create the binary."

ENV DEBIAN_FRONTEND=noninteractive

ARG PROFILE=release
WORKDIR /subsocial

RUN apt-get update && \
	apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
	apt-get install -y cmake cmake pkg-config libssl-dev git clang libclang-dev

COPY scripts /subsocial

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin" && \
	./init.sh

COPY . /subsocial

RUN ~/.cargo/bin/cargo build "--$PROFILE"

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.10.2
LABEL maintainer="dappforce@pm.me"
LABEL description="This is the 2nd stage: a very small image where we copy the Subsocial PC Node binary."
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	useradd -m -u 1000 -U -s /bin/sh -d /subsocial subsocial

COPY --from=builder /subsocial/target/$PROFILE/subsocial-collator /usr/local/bin

# checks
RUN ldd /usr/local/bin/subsocial-collator && \
	/usr/local/bin/subsocial-collator --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /usr/bin /usr/sbin /usr/share/man

## Copy chainspecs into the image
COPY res/rococo-local.json /
COPY res/rococo.json /

RUN mkdir /data
VOLUME ["/data"]

USER subsocial
EXPOSE 30333 9933 9944

ENTRYPOINT ["/usr/local/bin/subsocial-collator"]
