# Using this stage to build the binary
FROM --platform=$BUILDPLATFORM buildpack-deps:buster as builder
RUN apt-get update && apt-get install --no-install-recommends -y \
    'curl' 'ca-certificates' 'lzip' && \
    rm -rf /var/lib/apt/lists/*

RUN mkdir -p /dist/usr/bin /dist/etc /dist/plugin
WORKDIR /dist

# Pull artifact for the version and platform
ARG CSIVERSION="v0.0.1-alpha"
ARG TARGETPLATFORM
SHELL ["/bin/bash", "-c"]
RUN PF_NOPREFIX=$(echo -n ${TARGETPLATFORM#*\/}) ; \
    PF_TAG=$(echo -n ${PF_NOPREFIX//\//}) ; \
    curl -sL "https://github.com/metal-csi/metal-csi/releases/download/${CSIVERSION}/metal-csi.${PF_TAG}.lz" | lzip -d > "/dist/usr/bin/metal-csi"

# Finalize dist directory
RUN chmod +x /dist/usr/bin/metal-csi
COPY dist.config.yml /dist/etc/metal-csi.yml

# # Target container
FROM debian:buster-slim
RUN apt-get update && apt-get install --no-install-recommends -y 'ca-certificates' && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /dist/ /
ENTRYPOINT ["/usr/bin/metal-csi"]
