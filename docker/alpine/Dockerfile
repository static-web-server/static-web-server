FROM --platform=$BUILDPLATFORM alpine:3.22.3 AS build

ARG TARGETPLATFORM
ARG SERVER_VERSION=0.0.0
ENV SERVER_VERSION=${SERVER_VERSION}

RUN set -eux \
    && apk --no-cache add \
        ca-certificates \
        tzdata \
        file \
    && true

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

FROM alpine:3.22.3

ARG SERVER_VERSION=0.0.0
ARG SERVER_USER_NAME=sws
ARG SERVER_USER_ID=1000
ARG SERVER_GROUP_NAME=sws
ARG SERVER_GROUP_ID=1000

ENV SERVER_VERSION=$SERVER_VERSION
ENV SERVER_USER_ID=$SERVER_USER_ID
ENV SERVER_USER_NAME=$SERVER_USER_NAME
ENV SERVER_GROUP_ID=$SERVER_GROUP_ID
ENV SERVER_GROUP_NAME=$SERVER_GROUP_NAME

LABEL version="${SERVER_VERSION}" \
    description="A cross-platform, high-performance and asynchronous web server for static files-serving." \
    maintainer="Jose Quintana <joseluisq.net>"

RUN set -eux \
    && addgroup -g $SERVER_GROUP_ID $SERVER_GROUP_NAME \
    && adduser -D -u $SERVER_USER_ID -G $SERVER_GROUP_NAME $SERVER_USER_NAME \
    && true

RUN set -eux \
    && apk --no-cache add \
        ca-certificates \
        tzdata \
    && true

RUN set -eux \
    && mkdir -p /home/$SERVER_USER_NAME/public \
    && chown -R $SERVER_USER_NAME:$SERVER_GROUP_NAME /home/$SERVER_USER_NAME \
    && ln -s /home/$SERVER_USER_NAME/public /var/public \
    && chown -R $SERVER_USER_NAME:$SERVER_GROUP_NAME /var/public \
    && true

USER $SERVER_USER_NAME:$SERVER_GROUP_NAME

COPY --from=build --chown=$SERVER_USER_NAME:$SERVER_GROUP_NAME \
    /usr/local/bin/static-web-server /usr/local/bin/static-web-server
COPY --chown=$SERVER_USER_NAME:$SERVER_GROUP_NAME \
    ./docker/alpine/entrypoint.sh /usr/local/bin/entrypoint.sh
COPY --chown=$SERVER_USER_NAME:$SERVER_GROUP_NAME \
    ./docker/public /home/$SERVER_USER_NAME/public

WORKDIR /home/$SERVER_USER_NAME

EXPOSE 80

STOPSIGNAL SIGQUIT

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

CMD ["static-web-server"]

# Metadata
LABEL org.opencontainers.image.vendor="Jose Quintana" \
    org.opencontainers.image.url="https://github.com/static-web-server/static-web-server" \
    org.opencontainers.image.title="Static Web Server" \
    org.opencontainers.image.description="A cross-platform, high-performance and asynchronous web server for static files-serving." \
    org.opencontainers.image.version="${SERVER_VERSION}" \
    org.opencontainers.image.documentation="https://static-web-server.net"
