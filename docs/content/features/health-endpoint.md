# Health endpoint

SWS provides an optional `/health` endpoint that can be used to check if it is running properly.
When the  `/health` is requested, SWS will generate a log only at the `debug` level instead of the usual `info` level for a regular file.

The HTTP methods supported are `GET` and `HEAD`.

This feature is disabled by default and can be controlled by the boolean `--health` option or the equivalent [SERVER_HEALTH](./../configuration/environment-variables.md#health) env.

## Usage with Kubernetes liveness probe

The health endpoint is well suited for the Kubernetes liveness probe:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: frontend
spec:
  containers:
    - name: sws
      image: frontend:1.0.0
      command:
        - static-web-server
        - --root=/public
        - --log-level=info
        - --health
      ports:
      - containerPort: 80
        name: http
      livenessProbe:
        httpGet:
          path: /health
          port: http
```
