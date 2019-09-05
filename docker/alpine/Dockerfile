FROM alpine:3.10

LABEL maintainer="Jose Quintana <joseluisq.net>"

RUN apk --no-cache add ca-certificates tzdata
RUN set -ex; \
    apkArch="$(apk --print-arch)"; \
    case "$apkArch" in \
    armhf) arch='armv6' ;; \
    aarch64) arch='arm64' ;; \
    x86_64) arch='amd64' ;; \
    *) echo >&2 "error: unsupported architecture: $apkArch"; exit 1 ;; \
    esac;

COPY ./bin/static-web-server /usr/local/bin/
COPY ./public /public
EXPOSE 80
CMD ["static-web-server"]

# Metadata
LABEL org.opencontainers.image.vendor="Jose Quintana" \
    org.opencontainers.image.url="https://git.joseluisq.net/joseluisq/static-web-server" \
    org.opencontainers.image.title="Static Web Server" \
    org.opencontainers.image.description="A blazing fast web server to static files-serving powered by Rust." \
    org.opencontainers.image.version="1.0.0-beta.2" \
    org.opencontainers.image.documentation="https://git.joseluisq.net/joseluisq/static-web-server"
