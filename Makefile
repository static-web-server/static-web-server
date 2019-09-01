PKG_NAME=static-web-server
PKG_TARGET=$(PKG_TARGET)
PKG_BIN_PATH=./bin

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
	@rustup target add $(PKG_TARGET)
	@cargo install --force cargo-make
	@cargo install cargo-audit
.PHONY: install

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
	@sudo chown -R rust:rust ./
	@cargo build --release
.PHONY: release

optimize:
	@mkdir -p $(PKG_BIN_PATH)
	@cp -rf ./target/$(PKG_TARGET)/release/$(PKG_NAME) $(PKG_BIN_PATH)
	@echo "Size before:"
	@du -sh $(PKG_BIN_PATH)/$(PKG_NAME)
	@strip $(PKG_BIN_PATH)/$(PKG_NAME)
	@echo "Size after:"
	@du -sh $(PKG_BIN_PATH)/$(PKG_NAME)
.PHONY: optimize

docker.image:
	@cargo make --makefile Tasks.Prod.toml docker_image
.PHONY: docker.image

load_test:
	@cargo make --makefile Tasks.Dev.toml loadtest
.PHONY: load_test
