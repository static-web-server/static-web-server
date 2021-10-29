ARG SERVER_VERSION=0.0.0

FROM joseluisq/static-web-server:${SERVER_VERSION}-alpine AS latest

FROM scratch

ARG SERVER_VERSION=0.0.0
ENV SERVER_VERSION=${SERVER_VERSION}

LABEL version="${SERVER_VERSION}" \
    description="A blazing fast and asynchronous web server for static files-serving." \
    maintainer="Jose Quintana <joseluisq.net>"

COPY --from=latest /usr/local/bin/static-web-server /
COPY ./docker/public /public

EXPOSE 80

STOPSIGNAL SIGQUIT

ENTRYPOINT ["/static-web-server"]

# Metadata
LABEL org.opencontainers.image.vendor="Jose Quintana" \
    org.opencontainers.image.url="https://github.com/joseluisq/static-web-server" \
    org.opencontainers.image.title="Static Web Server" \
    org.opencontainers.image.description="A blazing fast and asynchronous web server for static files-serving." \
    org.opencontainers.image.version="${SERVER_VERSION}" \
    org.opencontainers.image.documentation="https://github.com/joseluisq/static-web-server"
