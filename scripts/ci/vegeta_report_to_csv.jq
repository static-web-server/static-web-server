[$name, (.latencies["min", "mean", "50th", "90th", "95th", "99th", "max"] | . / 1000 | round / 1000)] | @csv
