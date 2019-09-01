FROM scratch

LABEL maintainer="Jose Quintana <joseluisq.net>"

COPY ./bin/static-web-server /

EXPOSE 80

VOLUME ["/tmp"]

ENTRYPOINT ["/static-web-server"]

# Metadata
LABEL org.opencontainers.image.vendor="Jose Quintana" \
    org.opencontainers.image.url="https://git.joseluisq.net/joseluisq/static-web-server" \
    org.opencontainers.image.title="Static Web Server" \
    org.opencontainers.image.description="A blazing fast web server to static files-serving powered by Rust." \
    org.opencontainers.image.version="1.0.0-beta.1" \
    org.opencontainers.image.documentation="https://git.joseluisq.net/joseluisq/static-web-server"
