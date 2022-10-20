# SWS - Benchmarks 2022

> A benchmark suite which measures the requests per second and latency in average for several web servers.

<img title="SWS - Benchmarks 2022" src="data/sws_benchmarks.png" width="860">

## How to use

Change `WRK_URL` with the corresponding server URL to export the wrk metrics.

```sh
WRK_URL="http://localhost" make wrk
```

## System

- **OS:** Arch Linux
- **Kernel:** 5.19.13-arch1-1 (64 bits)
- **Processor:** 4 × Intel® Core™ i7-6500U
- **RAM:** 8 GiB

## Data

For data used see [data](./data/) directory for more details.
