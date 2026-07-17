ARG SERVER_VERSION=0.0.0

FROM ghcr.io/static-web-server/static-web-server:${SERVER_VERSION}-fips-alpine AS build

FROM scratch

ARG SERVER_VERSION=0.0.0
ENV SERVER_VERSION=${SERVER_VERSION}

LABEL version="${SERVER_VERSION}" \
    description="A cross-platform, high-performance and asynchronous web server for static files-serving. (FIPS build)" \
    maintainer="Jose Quintana <joseluisq.net>"

COPY --from=build /usr/local/bin/static-web-server /
COPY ./docker/public /public

EXPOSE 8787

STOPSIGNAL SIGQUIT

ENTRYPOINT ["/static-web-server"]

# Metadata
LABEL org.opencontainers.image.vendor="Jose Quintana" \
    org.opencontainers.image.url="https://github.com/static-web-server/static-web-server" \
    org.opencontainers.image.title="Static Web Server (FIPS)" \
    org.opencontainers.image.description="A cross-platform, high-performance and asynchronous web server for static files-serving. FIPS-validated TLS." \
    org.opencontainers.image.version="$SERVER_VERSION" \
    org.opencontainers.image.documentation="https://static-web-server.net"
