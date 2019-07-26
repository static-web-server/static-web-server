FROM scratch
COPY certs/ca-certificates.crt /etc/ssl/certs/
COPY static-web-server /
EXPOSE 80
VOLUME ["/tmp"]

ENTRYPOINT ["/static-web-server"]

# Metadata
LABEL org.opencontainers.image.vendor="Jose Quintana" \
    org.opencontainers.image.url="https://git.io/static-web-server" \
    org.opencontainers.image.title="Static Web Server" \
    org.opencontainers.image.description="A blazing fast web server to static files-serving powered by Rust." \
    org.opencontainers.image.version="$VERSION" \
    org.opencontainers.image.documentation="https://git.io/static-web-server"
