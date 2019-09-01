help:
	@echo
	@echo "Static Web Server"
	@echo "A fast web server to static files-serving powered by Rust Iron. 🚀"
	@echo
	@echo "Development:"
	@echo "Please use \`make <target>\` where <target> is one of:"
	@echo "    install           to install dependencies."
	@echo "    run               to run server in development."
	@echo "    watch             to run server (watch files mode) in development."
	@echo "    release           to build a release."
	@echo "    docker_image      to build a Docker image."
	@echo

install:
	@rustup target add x86_64-unknown-linux-musl
	@cargo install --force cargo-make
.PHONY: install

optimize:
	-mkdir -p ./bin
	-cp -rf ./target/x86_64-unknown-linux-musl/release/static-web-server ./bin
	-echo "Size before:"
	-du -sh ./bin/static-web-server
	-strip ./bin/static-web-server
	-echo "Size after:"
	-du -sh ./bin/static-web-server
.PHONY: optimize

run:
	@cargo make --makefile Tasks.Dev.toml run
.PHONY: run

test:
	@echo "There are no tests at the moment!"
.PHONY: test

watch:
	@cargo make --makefile Tasks.Dev.toml watch
.PHONY: watch

release:
	@cargo make --makefile Tasks.Prod.toml release
.PHONY: release

docker.image:
	@cargo make --makefile Tasks.Prod.toml docker_image
.PHONY: docker.image

load_test:
	@cargo make --makefile Tasks.Dev.toml loadtest
.PHONY: load_test
