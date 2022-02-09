# Platforms & Architectures

Currently only the following platforms/architectures are supported.

!!! info "Docker"
    For Docker images supported list see [Docker OS/Arch](/features/docker/#osarch).

## Linux

#### x86
  - `i686-unknown-linux-gnu` 
  - `i686-unknown-linux-musl`

#### x86_64
  - `x86_64-unknown-linux-gnu`
  - `x86_64-unknown-linux-musl`

#### ARM
  - `arm-unknown-linux-gnueabihf` (armv6)
  - `arm-unknown-linux-musleabihf` (armv6)
  - `armv7-unknown-linux-musleabihf` (armv7)

#### ARM64
  - `aarch64-unknown-linux-musl`
  - `aarch64-unknown-linux-gnu`

## macOS

#### x86_64
  - `x86_64-apple-darwin`

#### ARM64
  - `aarch64-apple-darwin`

## Windows

#### x86
  - `i686-pc-windows-msvc`

#### x86_64
  - `x86_64-pc-windows-msvc`
  - `x86_64-pc-windows-gnu`

#### ARM64
  - ~~`aarch64-pc-windows-msvc`~~ (temporarily disabled until [briansmith/ring#1167](https://github.com/briansmith/ring/issues/1167))

## FreeBSD

#### x86
  - `i686-unknown-freebsd`

#### x86_64
  - `x86_64-unknown-freebsd`
