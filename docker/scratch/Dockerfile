FROM scratch

COPY ./bin/static-web-server /

EXPOSE 80

VOLUME ["/tmp"]

ENTRYPOINT ["/static-web-server"]

# Metadata
LABEL org.opencontainers.image.vendor="Jose Quintana" \
    org.opencontainers.image.url="https://git.joseluisq.net/joseluisq/static-web-server" \
    org.opencontainers.image.title="Static Web Server" \
    org.opencontainers.image.description="A blazing fast web server to static files-serving powered by Rust." \
    org.opencontainers.image.version="0.0.0" \
    org.opencontainers.image.documentation="https://git.joseluisq.net/joseluisq/static-web-server"
