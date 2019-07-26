help:
	@echo
	@echo "Static Web Server"
	@echo "Web Server to static file-serving."
	@echo
	@echo "Please use \`make <target>\` where <target> is one of:"
	@echo "    install           to install dependencies."
	@echo "    run               to run server in development."
	@echo "    watch             to run server (watch files mode) in development."
	@echo "    release           to build a release."
	@echo "    docker_image      to build a Docker image."
	@echo

install:
	@cargo install --force cargo-make
	@cargo install cargo-release
.PHONY: install

run:
	@cargo make --makefile Tasks.Dev.toml run
.PHONY: run

watch:
	@cargo make --makefile Tasks.Dev.toml watch
.PHONY: watch

release:
	@cargo make --makefile Tasks.Prod.toml release
.PHONY: release

docker_image:
	@cargo make --makefile Tasks.Prod.toml docker_image
.PHONY: docker_image

load_test:
	@cargo make --makefile Tasks.Dev.toml loadtest
.PHONY: load_test
