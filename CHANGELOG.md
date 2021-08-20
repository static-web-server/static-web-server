# Static Web Server v1 - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

_**Note:** See changelog for v2 under the [master](https://github.com/joseluisq/static-web-server/blob/master/CHANGELOG.md) branch._

## v1.18.0 - 2021-08-20

__Updates__

- [806a276](https://github.com/joseluisq/static-web-server/commit/806a276) Update dependencies including OpenSSL, httparse and related crates.

__Refactorings__

- [c9e7222](https://github.com/joseluisq/static-web-server/commit/c9e7222) Cross-platform `ctrl-c` signal handling including Windows support.
- [3c7d9f6](https://github.com/joseluisq/static-web-server/commit/3c7d9f6) Remove needless borrow on server module.

__Docs__

- [5a14787](https://github.com/joseluisq/static-web-server/commit/5a14787) Changelog support for v1. PR [#50](https://github.com/joseluisq/static-web-server/pull/50)

## v1.17.1 - 2021-06-30

__Fixes__

- [f976586](https://github.com/joseluisq/static-web-server/commit/f976586) Fix static file base path not resolved (canonicalized) properly. For example when uri paths map to symlinks on file system.
- [35d5a1e](https://github.com/joseluisq/static-web-server/commit/35d5a1e) Fix missing server version env of "scratch" Dockerfile.

__Updates__

- [1fc9d7f](https://github.com/joseluisq/static-web-server/commit/1fc9d7f) Update dependencies 30.06.2021.

__Docs__

-  [8979292](https://github.com/joseluisq/static-web-server/commit/8979292) Describe current supported targets.

__Refactorings__

- [34cd35b](https://github.com/joseluisq/static-web-server/commit/34cd35b) Minor config project updates.

## v1.17.0 - 2021-06-21

__Updates__

- [54ae2d9](https://github.com/joseluisq/static-web-server/commit/54ae2d9) Update dependencies which includes OpenSSL crate updates and others (also [299e01c](https://github.com/joseluisq/static-web-server/commit/299e01c)).

__Features__

-  [f78de5a](https://github.com/joseluisq/static-web-server/commit/f78de5a) Additionally to current three targets `x86_64-unknown-linux-musl`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`. We are supporting six ones more, among which five ARMs stand out:
    - `x86_64-unknown-linux-gnu`
    - `aarch64-apple-darwin`
    - `aarch64-pc-windows-msvc`
    - `aarch64-unknown-linux-gnu`
    - `aarch64-unknown-linux-musl`
    - `arm-unknown-linux-gnueabihf`

Find those targets attached to this release.

__Refactorings__

- [bf87d7f](https://github.com/joseluisq/static-web-server/commit/bf87d7f) Improve error messages for address binding errors. PR [#43](https://github.com/joseluisq/static-web-server/pull/43)
- [842be1d](https://github.com/joseluisq/static-web-server/commit/842be1d) Update Docker files in order to get the new Linux binary source.

__Note about releases__

[Rust Nightly](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html) is powering the releases from now on the CI. This makes possible to reach more targets in the future.
For more details about it see [Rust Nightly targets supported](https://doc.rust-lang.org/nightly/rustc/platform-support.html).
However clarify that the `static-web-server` project is not using any nightly feature but only _**stable Rust**_ ones and also the project is tested against nightly and stable Rust on CI recurrently in order to be notified in case of _"regressions or bugs introduced in Nightly Rust"_. However it is [known](https://stackoverflow.com/a/56067977/2510591) that the nightly compiler is very stable therefore the reason why we have chosen it for release targets via CI like many other popular Rust projects.
In any case, don't hesitate to file an issue.

## v1.16.0 - 2021-05-26

__Updates__

- [b0593ff](https://github.com/joseluisq/static-web-server/commit/b0593ff) Binaries compiled with latest Rust [1.52.1](https://blog.rust-lang.org/2021/05/10/Rust-1.52.1.html) release.
- [c96cd48](https://github.com/joseluisq/static-web-server/commit/c96cd48) Update dependencies 26.05.2021

## v1.15.0 - 2021-04-20

__Features__

- [8cf7909](https://github.com/joseluisq/static-web-server/commit/8cf7909) Binaries compiled with latest Rust [1.51.0](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html) release.
- [cfbffb8](https://github.com/joseluisq/static-web-server/commit/cfbffb8) [Alpine 3.13](https://alpinelinux.org/posts/Alpine-3.13.0-released.html) Docker image.

__Updates__

- [12824ed](https://github.com/joseluisq/static-web-server/commit/12824ed) Update dependencies 20.04.2021 (also [996c181](https://github.com/joseluisq/static-web-server/commit/996c181), [c383b8a](https://github.com/joseluisq/static-web-server/commit/c383b8a)). Upgrades also include a OpenSSL security upgrade to `1.1.1k` which fixes `CVE-2021-3450`. More details on https://www.openssl.org/news/secadv/20210325.txt


__Docs__

- [304288c](https://github.com/joseluisq/static-web-server/commit/304288c) Minor README clarifications.

## v1.14.0 - 2021-02-14

__Features__

- [252406d](https://github.com/joseluisq/static-web-server/commit/252406d) Binaries compiled with latest Rust [v1.50.0](https://blog.rust-lang.org/2021/02/11/Rust-1.50.0.html) release.
- [feac82e](https://github.com/joseluisq/static-web-server/commit/feac82e) Directory listing support via new `-i, --directory-listing` boolean flag (equivalent `SERVER_DIRECTORY_LISTING` env variable) which is disabled by default. PR [#32](https://github.com/joseluisq/static-web-server/pull/32) resolves [#31](https://github.com/joseluisq/static-web-server/issues/31).
- [5026bfe](https://github.com/joseluisq/static-web-server/commit/5026bfe) Windows x86_64 binary support (.exe). See binary file attached to this release. PR [#33](https://github.com/joseluisq/static-web-server/pull/33) resolves [#29](https://github.com/joseluisq/static-web-server/issues/29).
- [116cbbf](https://github.com/joseluisq/static-web-server/commit/116cbbf) UTF-8 URL decoding support (also [cfcb6c](https://github.com/joseluisq/static-web-server/commit/cfcb6c) for directory listing)

__Refactorings__

- [4fe89c5](https://github.com/joseluisq/static-web-server/commit/4fe89c5) Trim CORS comma-separated allowed hosts (CORS server argument).
- [8733d2a](https://github.com/joseluisq/static-web-server/commit/8733d2a) Simplify public example asset files.
- [1c1c2af](https://github.com/joseluisq/static-web-server/commit/1c1c2af) Better 4xx/50x status code checks and more concise default error page content.
- [db51d77](https://github.com/joseluisq/static-web-server/commit/db51d77) `Option` type used for certain server arguments.
- [6551018](https://github.com/joseluisq/static-web-server/commit/6551018) Static file middleware searching optimizations which increases web server performance. 
- [a6cca08](https://github.com/joseluisq/static-web-server/commit/a6cca08) Improved static file path resolving.
- [5fed730](https://github.com/joseluisq/static-web-server/commit/5fed730) Improved error handling on directory listing feature.
- [d5e6bdd](https://github.com/joseluisq/static-web-server/commit/d5e6bdd) Improve error handling on static files module.
- [b167030](https://github.com/joseluisq/static-web-server/commit/b167030) Optimize few dependencies features which reduces binary size.
- [dd59cd3](https://github.com/joseluisq/static-web-server/commit/dd59cd3) Explicit OS target for signals handling (just Unix-like & Windows distinction)

__Updates__

- [a664866](https://github.com/joseluisq/static-web-server/commit/a664866) Update dependencies 14.02.2021

__Codebase__

- [533006f](https://github.com/joseluisq/static-web-server/commit/533006f) Refactor project modules structure & incorporate the static file middleware into the project.

__Docs__

- [6887624](https://github.com/joseluisq/static-web-server/commit/6887624) Remove whitespace description for CORS option.

## v1.13.0 - 2021-01-24

__Updates__

- [300337e](https://github.com/joseluisq/static-web-server/commit/300337e) Update dependencies 24.01.2021 (also [a72b31d](https://github.com/joseluisq/static-web-server/commit/a72b31d))

__Features__

- __Github Sponsor support__. Consider to support this project via [Github Sponsor](https://github.com/sponsors/joseluisq ) or Paypal [paypal.me/joseluisqs](https://paypal.me/joseluisqs).
- [63a05fa](https://github.com/joseluisq/static-web-server/commit/63a05fa) HTTP to HTTPS redirection support. PR [#26](https://github.com/joseluisq/static-web-server/pull/26) resolves [#25](https://github.com/joseluisq/static-web-server/issues/25) by [@HenningHolmDE](https://github.com/HenningHolmDE). See [README](https://github.com/joseluisq/static-web-server/tree/1.x#environment-variables) file for more details.
- [77b07be](https://github.com/joseluisq/static-web-server/commit/77b07be) CLI argument aliases for some options. See [README](https://github.com/joseluisq/static-web-server/tree/1.x#command-line-arguments) file for more details.

__Refactorings__

- [61d819d](https://github.com/joseluisq/static-web-server/commit/61d819d) Minor server log info tweaks.
- [d595219](https://github.com/joseluisq/static-web-server/commit/d595219) Prefer `&[]` instead of `Vec` ptr as arg type for `on_server_running` function.

__Codebase__

- [f3008f3](https://github.com/joseluisq/static-web-server/commit/f3008f3) Github Actions as new CI.

## v1.12.0 - 2021-01-02

__Updates__

- [c3583f3](https://github.com/joseluisq/static-web-server/commit/c3583f3) Binaries compiled with latest Rust [v1.49.0](https://blog.rust-lang.org/2020/12/31/Rust-1.49.0.html) release.
- [2732bd3](https://github.com/joseluisq/static-web-server/commit/2732bd3) Update dependencies 02.01.2021 (also [acef399](https://github.com/joseluisq/static-web-server/commit/acef399), [a7b3f40](https://github.com/joseluisq/static-web-server/commit/a7b3f40))

__Bug fixes__

- [c3583f3](https://github.com/joseluisq/static-web-server/commit/c3583f3) Server error response during large file downloads (Macos/Linux). Issue [#24](https://github.com/joseluisq/static-web-server/issues/24) reported by [@lukasa1993](https://github.com/lukasa1993).
The Rust 1.49.0 upgrade solves the issue which was [published on 1.48.0](https://github.com/rust-lang/rust/commit/db7ee7cc0ddf46a52f53a8a141cd0747f829999a)

__Features__

- [62c8fc2](https://github.com/joseluisq/static-web-server/commit/62c8fc2) Alpine 1.12 Docker image.

__Darwin__

- Binary compiled on Mac OS X SDK 10.15 with a minimum 10.14 version. [joseluisq/rust-linux-darwin-builder@v1.49.0](https://github.com/joseluisq/rust-linux-darwin-builder/releases/tag/v1.49.0)

__Binary sizes__

- **Linux:** Static binary size has no changed (__4.8M__) on current version `v1.12.0`.
- **Darwin:** Dynamic binary size was decreased (__0.1M__). From __1.7M__ (`v1.11.0`) to __1.6M__ (current version `v1.12.0`)

__Codebase__

- [7581c99](https://github.com/joseluisq/static-web-server/commit/7581c99) Ignore some project directories.

## v1.11.0 - 2020-10-15

__Updates__

- [d4e32ca](https://github.com/joseluisq/static-web-server/commit/d4e32ca) Binaries compiled with latest Rust [v1.47.0](https://blog.rust-lang.org/2020/10/08/Rust-1.47.html) release.
- [dd20c3e](https://github.com/joseluisq/static-web-server/commit/dd20c3e) Update dependencies 15.10.2020 (also [d4e32ca](https://github.com/joseluisq/static-web-server/commit/d4e32ca), [c4db7c6](https://github.com/joseluisq/static-web-server/commit/c4db7c6))

__Improvements__

- [6d3dc45](https://github.com/joseluisq/static-web-server/commit/6d3dc45) Shrink Darwin binary size. 
- [2401b24](https://github.com/joseluisq/static-web-server/commit/2401b24) Include a `SERVER_VERSION` (server release version) env on every Docker image (also [ee47f76](https://github.com/joseluisq/static-web-server/commit/ee47f76), [74040cf](https://github.com/joseluisq/static-web-server/commit/74040cf)).

__Binary sizes__

- **Linux:** Static binary size was increased (__0.5M__). From __4.3M__ (`v1.10.0`) to __4.8M__ (current version `v1.11.0`)
- **Darwin:** Dynamic binary size was decreased (__0.4M__). From __2.1M__ (`v1.10.0`) to __1.7M__ (current version `v1.11.0`)

__Codebase__

- [cbe1783](https://github.com/joseluisq/static-web-server/commit/cbe1783) Automate Docker images versioning with Drone pipelines.

## v1.10.0 - 2020-07-07

__Updates__

- [ff2f2ba](https://github.com/joseluisq/static-web-server/commit/ff2f2ba) Binaries compiled with latest Rust [v1.44.1](https://blog.rust-lang.org/2020/06/18/Rust.1.44.1.html) release.
- [9732d47](https://github.com/joseluisq/static-web-server/commit/9732d47) Update dependencies 07.07.2020 (also [eb30e11](https://github.com/joseluisq/static-web-server/commit/eb30e11))

__Improvements__

- [93aecf2](https://github.com/joseluisq/static-web-server/commit/93aecf2) Use `jemalloc` as global allocator on Linux Musl 64-bit systems. Which increases the web server performance (also [0fda53f](https://github.com/joseluisq/static-web-server/commit/0fda53f)). See more details on PR [#22](https://github.com/joseluisq/static-web-server/pull/22).  

__Linux binary size__

Static binary size was increased (__0.2M__). From __4.1M__ (`v1.9.2`) to __4.3M__ (current version `v1.10.0`)

__Codebase__

- [8e5a3c7](https://github.com/joseluisq/static-web-server/commit/8e5a3c7) Refactor project dev structure.

## v1.9.2 - 2020-06-08

__Bugfixes__

- [034263e](https://github.com/joseluisq/static-web-server/commit/034263e) Fix regression introduced by commit c0d2891 which displays wrong content type `text/plain` instead of `text/html` for 404/50x error pages.

## v1.9.1 - 2020-06-08

__Updates__

- [d87dafd](https://github.com/joseluisq/static-web-server/commit/d87dafd) Binaries compiled with latest Rust [v1.43.1](https://blog.rust-lang.org/2020/05/07/Rust.1.43.1.html) release.
- [0d7038f](https://github.com/joseluisq/static-web-server/commit/0d7038f) Update dependencies 08.06.20 (also [044b4a9](https://github.com/joseluisq/static-web-server/commit/044b4a9))

__Improvements__

- Update `iron-staticfile-middleware` to [v0.4.2](https://github.com/joseluisq/iron-staticfile-middleware/releases/tag/v0.4.2) which increases slightly performance on every check of text-based mime types array.

__Refactors__

- [c0d2891](https://github.com/joseluisq/static-web-server/commit/c0d2891) Prefer `const` for the default content type on Error Page middeware.
- [c3a7d73](https://github.com/joseluisq/static-web-server/commit/c3a7d73) Minor syntax improvements on the Gzip middeware and the Staticfile handler.
- [f7564f6](https://github.com/joseluisq/static-web-server/commit/f7564f6) Remove unnecessary `to_string` conversion on server address.
- [23a6a15](https://github.com/joseluisq/static-web-server/commit/23a6a15) Remove git tag `latest` of release tag target.

## v1.9.0 - 2020-05-09

__Features__

-  Binaries compiled with latest Rust [v1.43.0](https://blog.rust-lang.org/2020/04/23/Rust-1.43.0.html) and OpenSSL `v1.1.1g`. More details on [rust-linux-darwin-builder](https://github.com/joseluisq/rust-linux-darwin-builder/releases/tag/v1.43.0) release.
- [b2d2f40](https://github.com/joseluisq/static-web-server/commit/b2d2f40) Partial Content Delivery support. Which allows to serve bytes of large files. Resolves [#15](https://github.com/joseluisq/static-web-server/issues/15).
- [1af7d28](https://github.com/joseluisq/static-web-server/commit/1af7d28) CORS support via `--cors-allow-origins` option and its equivalent env. Resolves [#18](https://github.com/joseluisq/static-web-server/issues/18).
- [ed94c4b](https://github.com/joseluisq/static-web-server/commit/ed94c4b) Enable Gzip compression only for known text-based file types. View file content types on issue [#19](https://github.com/joseluisq/static-web-server/issues/19).
- [77bf538](https://github.com/joseluisq/static-web-server/commit/77bf538) Update dependencies 10.05.2020 (also [b2d2f40](https://github.com/joseluisq/static-web-server/commit/b2d2f40), [7a0fed0](https://github.com/joseluisq/static-web-server/commit/7a0fed0))

__Linux binary size__

Static binary size was increased (__0.1M__). From __4.0M__ (`v1.8.0`) to __4.1M__ (current version `v1.9.0`)


__Documentation__

- [e37d513](https://github.com/joseluisq/static-web-server/commit/e37d513) Corresponding documentation updates reflecting changes and features.

__Codebase__

- [1b1174d](https://github.com/joseluisq/static-web-server/commit/1b1174d) Prefer const over static on default error page content.
- [4bda5f3](https://github.com/joseluisq/static-web-server/commit/4bda5f3) Simplify Gzip middleware.
- [9a109d7](https://github.com/joseluisq/static-web-server/commit/9a109d7) Staging CI pipeline.

## v1.8.0 - 2020-04-21

__Overview__

This minor release `v1.8.0` introduces the following:

- `assets` directory path configurable and independent (no more relative to the `root`). This contains a **"breaking change"**. (Please see below).
- `assets` directory route is always the directory name. That means if you have `/public/my-assets` then the route will be `/my-assets`. 
- New option for configurable logging levels. `error` log level by default. (More details below).
* Reply with an empty response body on HEAD requests.
* Skip Gzip compression on HEAD requests.
* `root` and `assets` directory paths checking before to start the server.
* Improve logging server information.

Please view the details involved about this minor release.

__Features__

* [2f7d042](https://github.com/joseluisq/static-web-server/commit/2f7d042) - Configurable assets directory. (minor breaking change).
* [0dd2abe](https://github.com/joseluisq/static-web-server/commit/0dd2abe) - Configurable logging levels (resolves [#16](https://github.com/joseluisq/static-web-server/issues/16)) via `--log-level` option or its equivalent env. Now `error` log level is enabled by default. Use `info` level if you want requests details. Check out the documentation for more options.
* [0a9f66b](https://github.com/joseluisq/static-web-server/commit/0a9f66b) - Update [`iron_staticfile_middleware`](https://github.com/joseluisq/iron-staticfile-middleware) to `v0.3.0`.
* [925c58a](https://github.com/joseluisq/static-web-server/commit/925c58a) - Update dependencies (April 21th 2020).

__Breaking change__

This minor release contains one "breaking change" due to the feature _"configurable assets directory"_ ([2f7d042](https://github.com/joseluisq/static-web-server/commit/2f7d042)).

__1. Configurable assets directory__

This feature makes `assets` directory (`--assets` option and its equivalent `SERVER_ASSETS` env) configurable and independent.

It means that `assets` directory is no more restricted to be relative to `root` directory. So absolute `paths` work now.

__1.1. Caveat and solution for relative assets directory__

If you are using an `assets` directory path relative to `root` (`v1.7.0` or early). For example if you have a root `./my-root` and assets `./my-assets`  directories. Just adjust your `assets` directory path to `./my-root/my-assets` or use another absolute path if you want.

This is necessary because from now, the `assets` directory is treated as an independent path. So an absolute path is recomendable for both directories (root and assets).

Otherwise, if you are using the default values provided by the server, you don't need to modify anything. Unless you had specified them yourself. If so, please proceed as explained above.

__Refactorings__

* [59777d7](https://github.com/joseluisq/static-web-server/commit/59777d7) - Empty response body on HEAD requests (also 07bf49b).
* [57184a5](https://github.com/joseluisq/static-web-server/commit/57184a5) - Skip Gzip compression on HEAD requests.
* [e40016d](https://github.com/joseluisq/static-web-server/commit/e40016d) - Root and assets directory paths checking.
* [597fa97](https://github.com/joseluisq/static-web-server/commit/597fa97) - Improve logging server information.

__Documentation__

New options and modifications details were updated, please take a look the documentation on README file.

* [8a5cf79](https://github.com/joseluisq/static-web-server/commit/8a5cf79) - Clarify assets default option value.
* [0767071](https://github.com/joseluisq/static-web-server/commit/0767071) - Docker stack notes about assets directory functionality.

__Codebase__

* [a449eb3](https://github.com/joseluisq/static-web-server/commit/a449eb3) - Refactor path helpers implementation.
* [2e2a6ac](https://github.com/joseluisq/static-web-server/commit/2e2a6ac) - Simplify error page implementation.
* [ff4ec6e](https://github.com/joseluisq/static-web-server/commit/ff4ec6e) - Static file unit test cases.
* [48b92ef](https://github.com/joseluisq/static-web-server/commit/48b92ef) - Skip files of Docker build context.

## v1.7.0 - 2020-04-04

__Features__

- [052c18d](https://github.com/joseluisq/static-web-server/commit/052c18d) Binaries compiled with latest Rust [v1.42.0](https://blog.rust-lang.org/2020/03/12/Rust-1.42.html)
- [4ae5b1a](https://github.com/joseluisq/static-web-server/commit/4ae5b1a) Update dependencies (April 4th 2020)

__Linux binary size__

Static binary size was increased (__0.1M__). From __3.9M__ (`v1.6.0`) to __4.0M__ (current version `v1.7.0`)

__Codebase__
- [18a6cd5](https://github.com/joseluisq/static-web-server/commit/18a6cd5) - Simplify makefile build tagets. Includes checksum task (also [1c3abdf](https://github.com/joseluisq/static-web-server/commit/1c3abdf))
- Among other project structural improvements

## v1.6.0 - 2020-03-04

__Features__

- [d0e7a7f](https://github.com/joseluisq/static-web-server/commit/d0e7a7f) Binaries compiled with latest Rust [v1.41.1](https://blog.rust-lang.org/2020/02/27/Rust-1.41.1.html).
- [a2a4e98](https://github.com/joseluisq/static-web-server/commit/a2a4e98) Signals termination support. View feature issue  [#13](https://github.com/joseluisq/static-web-server/issues/13) resolved by PR [#14](https://github.com/joseluisq/static-web-server/pull/14).
- [08d5847](https://github.com/joseluisq/static-web-server/commit/08d5847) Docker Alpine image upgraded to latest `v3.11`.
- [20b9fd2](https://github.com/joseluisq/static-web-server/commit/20b9fd2) Docker entrypoint shell script for Alpine image. It gives the ability to pass flag arguments on a `docker run` execution or via the `command` option in a `docker-compose` file. Example: `docker run --rm static-web-server:alpine --help`. Flag arguments (with dashes) will be passed to `static-web-server`, otherwise they will be treated as shell commands. Example of an interactive run: `docker run --rm -it static-web-server:alpine sh`.
- [881f37c](https://github.com/joseluisq/static-web-server/commit/881f37c) Dependency updates 03.03.2020. (also [a96e031](https://github.com/joseluisq/static-web-server/commit/a96e031), [c0e96e8](https://github.com/joseluisq/static-web-server/commit/c0e96e8))

__Linux binary size__

Static binary size was reduced (__0.3M__). From __4.2M__ (`v1.5.0`) to __3.9M__ (current version `v1.6.0`)


__Codebase__

- [bac3914](https://github.com/joseluisq/static-web-server/commit/bac3914) `openssl` as dev-dependency vendored.
- [27dd9ff](https://github.com/joseluisq/static-web-server/commit/27dd9ff) Update makefile cross-compiling tasks using `rust-linux-darwin-builder` (also [38808fb](https://github.com/joseluisq/static-web-server/commit/38808fb)).
- [478d100](https://github.com/joseluisq/static-web-server/commit/478d100) Rename Docker file templates. (also [ba105e4](https://github.com/joseluisq/static-web-server/commit/ba105e4))
- [2adc5db](https://github.com/joseluisq/static-web-server/commit/2adc5db) Relocate Docker templates version script.
- [34e0c59](https://github.com/joseluisq/static-web-server/commit/34e0c59) Minor documentation updates.


## v1.5.0 - 2020-02-02

__Features__

- [6dc5056](https://github.com/joseluisq/static-web-server/commit/6dc5056) Feat: gzip compression on demand via `accept-encoding` header (PR [#12](https://github.com/joseluisq/static-web-server/pull/12) resolves [#10](https://github.com/joseluisq/static-web-server/issues/10)). 
- [eaebf82](https://github.com/joseluisq/static-web-server/commit/eaebf82) Feat: Update dependencies Feb 02, 2020. It also resolves [#11](https://github.com/joseluisq/static-web-server/issues/11) `head` request method feature.
- 
## v1.4.0 - 2020-01-15

__Features__

- [d195e74](https://github.com/joseluisq/static-web-server/commit/d195e74) Feat: Add TLS/SSL support (PR [#9](https://github.com/joseluisq/static-web-server/pull/9) resolves [#2](https://github.com/joseluisq/static-web-server/issues/2)).  Introducing three new options such as `--tls`, `--tls-pkcs12` and `--tls-pkcs12-passwd` as well as their corresponding environment variables.
- [63f2c82](https://github.com/joseluisq/static-web-server/commit/63f2c82) Docs: Add TLS/SSL section. View [usage](https://github.com/joseluisq/static-web-server#usage) on readme file.
- [9c58c9c](https://github.com/joseluisq/static-web-server/commit/9c58c9c) Feat: Update to latest dependencies Jun 15, 2020.
- Due TLS/SSL feature the __Linux__ binary size has incremented (`2,3MB`) from `1.8MB` to `4.1MB`.

__Misc__

- [cf0fb53](https://github.com/joseluisq/static-web-server/commit/cf0fb53) Docs: Update new options usage in readme file
- [a7d55ad](https://github.com/joseluisq/static-web-server/commit/a7d55ad) Fix: Permissions for Rust project directory
- [86f8d8a](https://github.com/joseluisq/static-web-server/commit/86f8d8a) Feat: Add more sections to Cargo manifest file

## v1.3.0 - 2020-01-07

__Features__

- [9179bc3](https://github.com/joseluisq/static-web-server/commit/9179bc3) Add binary support for __Macos__ `x86_64-apple-darwin` thanks to [Rust Linux / Darwin Builder](https://github.com/joseluisq/rust-linux-darwin-builder). Resolves issue [#8](https://github.com/joseluisq/static-web-server/issues/8).
- [1509f5f](https://github.com/joseluisq/static-web-server/commit/1509f5f) Update dependencies Jan 7, 2020

__Misc__

- [92b04e1](https://github.com/joseluisq/static-web-server/commit/92b04e1) Add Linux and Darwin makefile tasks
- [a7a6d61](https://github.com/joseluisq/static-web-server/commit/a7a6d61) Fix tarball and shrinking makefile tasks
- [56c425a](https://github.com/joseluisq/static-web-server/commit/56c425a) Remove Github release step from Drone pipeline

## v1.2.0 - 2019-12-26

__Features__

-  [2979f7a](https://github.com/joseluisq/static-web-server/commit/2979f7a) Makes optional the _**error pages feature**_ introducing default response HTML messages for 400 and 50x errors. (PR [#7](https://github.com/joseluisq/static-web-server/pull/7) resolves [#6](https://github.com/joseluisq/static-web-server/issues/6)) 

__Refactors__

- [8a90486](https://github.com/joseluisq/static-web-server/commit/8a90486) Update pages and assets example
- [3e64cbb](https://github.com/joseluisq/static-web-server/commit/3e64cbb) Update server config information

__Misc__

- [593ca98](https://github.com/joseluisq/static-web-server/commit/593ca98) Minor documentation updates
- [9f63108](https://github.com/joseluisq/static-web-server/commit/9f63108) Disable Travis CI email notifications

## v1.1.0 - 2019-12-25

__Features__

- Rust [v1.40.0](https://blog.rust-lang.org/2019/12/19/Rust-1.40.0.html) ([73528e3](https://github.com/joseluisq/static-web-server/commit/73528e3) 2019-12-16)
- CLI options support. PR [#4](https://github.com/joseluisq/static-web-server/pull/4) by [@dlalic](https://github.com/dlalic) resolves [#1](https://github.com/joseluisq/static-web-server/issues/1). See README file for more details.
- Error pages support. PR [#5](https://github.com/joseluisq/static-web-server/pull/5) resolves [#3](https://github.com/joseluisq/static-web-server/issues/3). See README file for more details.
- Binary size increment (`0.4MB`) from `1.4MB` to `1.8MB`.

__Breaking changes__

Due __error HTML pages feature #5__ now the server requires two settings for display 404 and 50x errors.

For example if you use environment variables try to append following variables with the two paths of your HTML pages:

```sh
#  HTML file path for 404 errors
SERVER_ERROR_PAGE_404=./public/404.html
# HTML file path for 50x errors
SERVER_ERROR_PAGE_50X=./public/50x.html
```

Or via CLI options:

```
--page404 <page404>    HTML file path for 404 errors [env: SERVER_ERROR_PAGE_404=]  [default: ./public/404.html]
--page50x <page50x>    HTML file path for 50x errors [env: SERVER_ERROR_PAGE_50X=]  [default: ./public/50x.html]
```

__Refactors__
- [112a1f3](https://github.com/joseluisq/static-web-server/commit/112a1f3) Update dockefile templates metadata
- [b34624c](https://github.com/joseluisq/static-web-server/commit/b34624c) Dependency updates 
- [5e9abd4](https://github.com/joseluisq/static-web-server/commit/5e9abd4) Apply cargo fix --edition to entire project 
- [4712b9c](https://github.com/joseluisq/static-web-server/commit/4712b9c) Minor tweaks for Cargo make tasks
- [6fb65b2](https://github.com/joseluisq/static-web-server/commit/6fb65b2) Rename `env.rs` to `config.rs`.
- New updated fork dependency [iron-staticfile-middleware](https://github.com/joseluisq/iron-staticfile-middleware)
- Minor project code base improvements

## v1.0.0 - 2019-11-28

This is the first major release :zap:

__Features__

- Rust [v1.39.0](https://blog.rust-lang.org/2019/11/07/Rust-1.39.0.html) ([4560ea788](https://github.com/joseluisq/static-web-server/commit/4560ea788) 2019-11-04)
- Binary size reduction for this current release from `1.6MB` to `1.4MB`.

## v1.0.0-beta.4 - 2019-11-28

__Updates__

- [eb96054](https://github.com/joseluisq/static-web-server/commit/eb96054) Refactor project structure
- [eb96054](https://github.com/joseluisq/static-web-server/commit/eb96054) Update dependencies
- [eb96054](https://github.com/joseluisq/static-web-server/commit/eb96054) Support for latest Rust [v1.39.0](https://blog.rust-lang.org/2019/11/07/Rust-1.39.0.html) (`[4560ea7](https://github.com/joseluisq/static-web-server/commit/4560ea7)` 2019-11-04)

__Deployment__

- [2fd780e](https://github.com/joseluisq/static-web-server/commit/2fd780e) Fix Cargo package config file name and version extraction 
- [3a69088](https://github.com/joseluisq/static-web-server/commit/3a69088) Fix release tarball files generation

__Misc__

- Binary size reduction for this current release from `1.6M` to `1.4M`.

## v1.0.0-beta.3 - 2019-10-10
__Features__

- Add [Rust v1.38.0](https://blog.rust-lang.org/2019/09/26/Rust-1.38.0.html) ([625451e](https://github.com/joseluisq/static-web-server/commit/625451e) 2019-09-23) compiling support.

## v1.0.0-beta.2 - 2019-09-05
