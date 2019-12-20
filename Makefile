PKG_TARGET=x86_64-unknown-linux-musl
PKG_BIN_PATH=./bin

PKG_NAME=$(shell cat Cargo.toml | sed -n 's/name = "\([^}]*\)"/\1/p' | head -n1)
PKG_TAG=$(shell cat Cargo.toml | sed -n 's/version = "\([^}]*\)"/\1/p' | head -n1)


#######################################
############# Development #############
#######################################

install:
	@rustup target add $(PKG_TARGET)
	@cargo install --force cargo-make
	@cargo install cargo-audit
.PHONY: install

run:
	@cargo make --makefile Tasks.Dev.toml run
.PHONY: run

watch:
	@cargo make --makefile Tasks.Dev.toml watch
.PHONY: watch

build:
	@cargo build --release --target $(PKG_TARGET)
.PHONY: build


#######################################
########### Utility tasks #############
#######################################

test:
	@sudo chown -R rust:rust ./
	@echo "Testing application..."
	@rustc --version
	@cargo test
.PHONY: test

fmt:
	@cargo fix --edition
	@cargo fmt --all
.PHONY: fmt

docker.image.alpine:
	@docker build \
		--rm=true -f ./docker/alpine/Dockerfile \
		--build-arg SERVER_VERSION="alpine" -t ${PKG_NAME}:alpine . --pull=true
.PHONY: docker.image.alpine


#######################################
########## Production tasks ###########
#######################################

# Compile release binary 
define build_release =
	set -e
	set -u

	sudo chown -R rust:rust ./
	echo "Compiling application..."
	rustc --version
	cargo build --release --target $(PKG_TARGET)
	echo "Release build completed!"
endef

# Shrink a release binary size
define build_release_shrink =
	set -e
	set -u

	mkdir -p $(PKG_BIN_PATH)
	cp -rf ./target/$(PKG_TARGET)/release/$(PKG_NAME) $(PKG_BIN_PATH)
	echo "Size before:"
	du -sh $(PKG_BIN_PATH)/$(PKG_NAME)
	strip $(PKG_BIN_PATH)/$(PKG_NAME)
	echo "Size after:"
	du -sh $(PKG_BIN_PATH)/$(PKG_NAME)
	echo "Release size shrinking completed!"
endef

# Creates release files (tarballs, zipballs) 
define build_release_files =
	set -e
	set -u

	cd $(PKG_BIN_PATH) && \
		tar czvf $(PKG_NAME)-v$(PKG_TAG)-x86_64-$(PKG_TARGET).tar.gz $(PKG_NAME)
	du -sh ./*
	echo "Release tarball/zipball files created!"
endef

prod.release:
	set -e
	set -u

	echo "Building a release..."

	$(build_release)
	$(build_release_shrink)
	$(build_release_files)
.ONESHELL: prod.release

prod.release.build:
	@$(build_release)
.ONESHELL: prod.release.build

prod.release.shrink:
	@$(build_release_shrink)
.ONESHELL: prod.release.shrink

prod.release.files:
	@$(build_release_files)
.ONESHELL: prod.release.files

prod.release.tag:
	git tag -d latest
	git push --delete origin latest

	# Update docker files to latest tag per platform
	./scripts/version.sh v$(PKG_TAG)

	git add .
	git commit . -m "v$(PKG_TAG)"
	git tag latest
	git tag v$(PKG_TAG)
	git push
	git push origin v$(PKG_TAG)
	git push origin latest
.ONESHELL: prod.release.tag

prod.release.dockerfiles:
	# Update docker files to latest tag per platform
	./scripts/version.sh v$(PKG_TAG)
.ONESHELL: prod.release.dockerfiles
