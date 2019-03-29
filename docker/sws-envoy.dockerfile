FROM envoyproxy/envoy-alpine:latest

LABEL maintainer=https://git.io/joseluisq

RUN set -ex \
    && apk update && apk add --no-cache bash ca-certificates

ADD bin /bin/static-web-server
ADD public /public

COPY docker-entrypoint.sh /entrypoint.sh

RUN chmod u+x /bin/static-web-server \
    && chmod u+x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
