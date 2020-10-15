FROM scratch

ARG SERVER_VERSION=0.0.0
ENV SERVER_VERSION=${SERVER_VERSION}

LABEL version="${SERVER_VERSION}" \
    description="A blazing fast static files-serving web server powered by Rust Iron." \
    maintainer="Jose Quintana <joseluisq.net>"

COPY ./bin/static-web-server /
COPY ./public /public

EXPOSE 80
VOLUME ["/public"]
ENTRYPOINT ["/static-web-server"]

# Metadata
LABEL org.opencontainers.image.vendor="Jose Quintana" \
    org.opencontainers.image.url="https://github.com/joseluisq/static-web-server" \
    org.opencontainers.image.title="Static Web Server" \
    org.opencontainers.image.description="A blazing fast static files-serving web server powered by Rust Iron." \
    org.opencontainers.image.version="${SERVER_VERSION}" \
    org.opencontainers.image.documentation="https://github.com/joseluisq/static-web-server"
