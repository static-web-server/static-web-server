FROM envoyproxy/envoy-alpine:36f39c746eb7d03b762099b206403935b11972d8

LABEL maintainer=https://git.io/joseluisq

RUN set -ex \
    && apk update && apk add --no-cache bash ca-certificates

ADD bin /bin
ADD public /public
ADD docker/docker-entrypoint.sh /entrypoint.sh

RUN chmod u+x /bin/static-web-server \
    && chmod u+x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
