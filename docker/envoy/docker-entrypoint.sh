#!/bin/sh

set -e

/bin/static-web-server &
/usr/local/bin/envoy -c /etc/envoy-service.yaml --service-cluster ${SERVICE_NAME}
