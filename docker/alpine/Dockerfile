FROM alpine:3.15

ARG SERVER_VERSION=0.0.0
ENV SERVER_VERSION=${SERVER_VERSION}

LABEL version="${SERVER_VERSION}" \
    description="A blazing fast static files-serving web server powered by Rust Iron." \
    maintainer="Jose Quintana <joseluisq.net>"

RUN apk --no-cache add ca-certificates tzdata
RUN set -ex; \
	wget --quiet -O /tmp/static-web-server.tar.gz "https://github.com/joseluisq/static-web-server/releases/download/v$SERVER_VERSION/static-web-server-v$SERVER_VERSION-x86_64-unknown-linux-musl.tar.gz"; \
    tar xzvf /tmp/static-web-server.tar.gz; \
    cp static-web-server-v${SERVER_VERSION}-x86_64-unknown-linux-musl/static-web-server /usr/local/bin/; \
	rm -rf /tmp/static-web-server.tar.gz static-web-server-v${SERVER_VERSION}-x86_64-unknown-linux-musl; \
	chmod +x /usr/local/bin/static-web-server

COPY ./docker/alpine/entrypoint.sh /
COPY ./public /public

EXPOSE 80
VOLUME ["/public"]
ENTRYPOINT ["/entrypoint.sh"]
CMD ["static-web-server"]

# Metadata
LABEL org.opencontainers.image.vendor="Jose Quintana" \
    org.opencontainers.image.url="https://github.com/joseluisq/static-web-server" \
    org.opencontainers.image.title="Static Web Server" \
    org.opencontainers.image.description="A blazing fast static files-serving web server powered by Rust Iron." \
    org.opencontainers.image.version="${SERVER_VERSION}" \
    org.opencontainers.image.documentation="https://github.com/joseluisq/static-web-server"
