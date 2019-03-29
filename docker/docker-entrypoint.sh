#!/bin/sh

set -e

case "$1" in
    *.yaml|*.yml) set -- static-web-server serve "$@" ;;
    serve|garbage-collect|help|-*) set -- static-web-server "$@" ;;
esac

exec "$@" &
/usr/local/bin/envoy -c /etc/envoy-service.yaml --service-cluster ${SERVICE_NAME}
