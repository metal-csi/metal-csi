# Using this stage to build the binary
FROM --platform=$BUILDPLATFORM buildpack-deps:buster as builder
RUN apt-get update && apt-get install --no-install-recommends -y \
    'curl' 'ca-certificates' 'lzip' && \
    rm -rf /var/lib/apt/lists/*

RUN mkdir -p /dist/usr/bin /dist/etc
WORKDIR /dist

# Pull artifact for the version and platform
ARG CSIVERSION="0.0.1-alpha"
ARG TARGETPLATFORM
SHELL ["/bin/bash", "-c"]
RUN PF_NOPREFIX=$(echo -n ${TARGETPLATFORM#*\/}) ; \
    PF_TAG=$(echo -n ${PF_NOPREFIX//\//}) ; \
    curl -sL "https://github.com/zed-csi/zed-csi/releases/download/v${CSIVERSION}/zed-csi.${PF_TAG}.lz" | lzip -d > "/dist/usr/bin/zed-csi"

# Finalize dist directory
RUN chmod +x /dist/usr/bin/zed-csi
COPY dist.config.yml /dist/etc/zed-csi.yml

# # Target container
FROM debian:buster-slim
RUN apt-get update && apt-get install --no-install-recommends -y 'ca-certificates' && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /dist/ /
ENTRYPOINT ["/usr/bin/zed-csi"]
