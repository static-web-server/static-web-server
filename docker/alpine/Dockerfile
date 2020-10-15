FROM alpine:3.11

ARG SERVER_VERSION=0.0.0
ENV SERVER_VERSION=${SERVER_VERSION}

LABEL version="${SERVER_VERSION}" \
    description="A blazing fast static files-serving web server powered by Rust Iron." \
    maintainer="Jose Quintana <joseluisq.net>"

RUN apk --no-cache add ca-certificates tzdata
RUN set -ex; \
    apkArch="$(apk --print-arch)"; \
    case "$apkArch" in \
    armhf) arch='armv6' ;; \
    aarch64) arch='arm64' ;; \
    x86_64) arch='amd64' ;; \
    *) echo >&2 "error: unsupported architecture: $apkArch"; exit 1 ;; \
    esac;

COPY ./docker/alpine/entrypoint.sh /
COPY ./bin/static-web-server /usr/local/bin/
COPY ./public /public
EXPOSE 80
ENTRYPOINT ["/entrypoint.sh"]
CMD ["static-web-server"]

# Metadata
LABEL org.opencontainers.image.vendor="Jose Quintana" \
    org.opencontainers.image.url="https://github.com/joseluisq/static-web-server" \
    org.opencontainers.image.title="Static Web Server" \
    org.opencontainers.image.description="A blazing fast static files-serving web server powered by Rust Iron." \
    org.opencontainers.image.version="${SERVER_VERSION}" \
    org.opencontainers.image.documentation="https://github.com/joseluisq/static-web-server"
