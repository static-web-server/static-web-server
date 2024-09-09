FROM --platform=$BUILDPLATFORM alpine:3.18.9 as build

ARG TARGETPLATFORM
ARG SERVER_VERSION=0.0.0
ENV SERVER_VERSION=${SERVER_VERSION}

RUN apk --no-cache add ca-certificates tzdata file

RUN set -ex; \
    case "$TARGETPLATFORM" in \
        "linux/amd64") target='x86_64-unknown-linux-musl' ;; \
        "linux/arm64") target='aarch64-unknown-linux-musl' ;; \
        "linux/386") target='i686-unknown-linux-musl' ;; \
        "linux/arm/v7") target='armv7-unknown-linux-musleabihf' ;; \
        "linux/arm/v6") target='arm-unknown-linux-musleabihf' ;; \
        *) echo >&2 "error: unsupported $TARGETPLATFORM architecture"; exit 1 ;; \
    esac; \
    wget --quiet -O /tmp/static-web-server.tar.gz "https://github.com/static-web-server/static-web-server/releases/download/v${SERVER_VERSION}/static-web-server-v${SERVER_VERSION}-${target}.tar.gz"; \
    tar xzvf /tmp/static-web-server.tar.gz; \
    cp static-web-server-v${SERVER_VERSION}-${target}/static-web-server /usr/local/bin/; \
    rm -rf /tmp/static-web-server.tar.gz static-web-server-v${SERVER_VERSION}-${target}; \
    chmod +x /usr/local/bin/static-web-server

RUN set -ex \
    && echo "Testing Docker image..." \
    && uname -a \
    && cat /etc/os-release \
    && static-web-server --version \
    && static-web-server --help \
    && file /usr/local/bin/static-web-server \
    && true

FROM alpine:3.18.9

ARG SERVER_VERSION=0.0.0
ENV SERVER_VERSION=${SERVER_VERSION}

LABEL version="${SERVER_VERSION}" \
    description="A cross-platform, high-performance and asynchronous web server for static files-serving." \
    maintainer="Jose Quintana <joseluisq.net>"

RUN apk --no-cache add ca-certificates tzdata

COPY --from=build /usr/local/bin/static-web-server /usr/local/bin/
COPY ./docker/alpine/entrypoint.sh /
COPY ./docker/public /public

EXPOSE 80

STOPSIGNAL SIGQUIT

ENTRYPOINT ["/entrypoint.sh"]

CMD ["static-web-server"]

# Metadata
LABEL org.opencontainers.image.vendor="Jose Quintana" \
    org.opencontainers.image.url="https://github.com/static-web-server/static-web-server" \
    org.opencontainers.image.title="Static Web Server" \
    org.opencontainers.image.description="A cross-platform, high-performance and asynchronous web server for static files-serving." \
    org.opencontainers.image.version="${SERVER_VERSION}" \
    org.opencontainers.image.documentation="https://github.com/static-web-server/static-web-server"
