#!/bin/sh

set -e

exec /bin/static-web-server &
/usr/local/bin/envoy -c /etc/envoy-service.yaml --service-cluster ${SERVICE_NAME}
