PKG_TARGET=x86_64-unknown-linux-musl
PKG_TARGET_DARWIN=x86_64-apple-darwin
RUST_VERSION=$(shell rustc --version | cut -d ' ' -f2)

PKG_BIN_PATH=./bin
PKG_TMP_PATH=/tmp

PKG_NAME=$(shell cat Cargo.toml | sed -n 's/name = "\([^}]*\)"/\1/p' | head -n1)
PKG_TAG=$(shell cat Cargo.toml | sed -n 's/version = "\([^}]*\)"/\1/p' | head -n1)

PKG_RELEASE_NAME=$(PKG_NAME)-v$(PKG_TAG)-$(PKG_TARGET)
PKG_RELEASE_NAME_DARWIN=$(PKG_NAME)-v$(PKG_TAG)-$(PKG_TARGET_DARWIN)

PKG_TMP_BIN_PATH=$(PKG_TMP_PATH)/$(PKG_RELEASE_NAME)
PKG_TMP_BIN_PATH_DARWIN=$(PKG_TMP_PATH)/$(PKG_RELEASE_NAME_DARWIN)

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

linux:
	@docker run --rm \
		--user rust:rust \
		--volume ${PWD}:/home/rust/static-web-server \
		--workdir /home/rust/static-web-server \
		joseluisq/rust-linux-darwin-builder:$(RUST_VERSION) \
		sh -c "rustc --version && \
			mkdir -p target bin && \
			cargo build --release --target $(PKG_TARGET) && \
			cp -rf ./target/$(PKG_TARGET)/release/$(PKG_NAME) $(PKG_BIN_PATH)/$(PKG_NAME)-linux && \
			strip $(PKG_BIN_PATH)/$(PKG_NAME)-linux && \
			du -sh $(PKG_BIN_PATH)/*"
.PHONY: linux

darwin:
	@docker run --rm \
		--user rust:rust \
		--volume ${PWD}:/home/rust/static-web-server \
		--workdir /home/rust/static-web-server \
		joseluisq/rust-linux-darwin-builder:$(RUST_VERSION) && \
		sh -c "rustc --version && \
			mkdir -p target bin && \
			cargo build --release --target $(PKG_TARGET_DARWIN) && \
			cp -rf ./target/$(PKG_TARGET_DARWIN)/release/$(PKG_NAME) $(PKG_BIN_PATH)/$(PKG_NAME)-darwin && \
			strip $(PKG_BIN_PATH)/$(PKG_NAME)-darwin && \
			du -sh $(PKG_BIN_PATH)/*"
.PHONY: darwin

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
	@cargo fix
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
	echo "Compiling release binary for $(PKG_TARGET)..."
	cargo build --release --target $(PKG_TARGET)
	echo
	echo "Compiling release binary for $(PKG_TARGET_DARWIN)..."
	cargo build --release --target $(PKG_TARGET_DARWIN)
	echo "Release builds completed!"
endef

# Shrink a release binary size
define build_release_shrink =
	set -e
	set -u

	echo "Copying release binaries..."

	mkdir -p $(PKG_BIN_PATH)

	# Linux
	mkdir -p $(PKG_TMP_BIN_PATH)
	cp -rf ./target/$(PKG_TARGET)/release/$(PKG_NAME) $(PKG_TMP_BIN_PATH)

	# Darwin
	mkdir -p $(PKG_TMP_BIN_PATH_DARWIN)
	cp -rf ./target/$(PKG_TARGET_DARWIN)/release/$(PKG_NAME) $(PKG_TMP_BIN_PATH_DARWIN)

	# Linux only
	echo "Performing binary shrinking for $(PKG_TARGET) release..."
	echo "Size before:"
	du -sh $(PKG_TMP_BIN_PATH)/$(PKG_NAME)
	strip $(PKG_TMP_BIN_PATH)/$(PKG_NAME)
	echo "Size after:"
	du -sh $(PKG_TMP_BIN_PATH)/$(PKG_NAME)
	echo "Copying $(PKG_TMP_BIN_PATH)/$(PKG_NAME) binary to $(PKG_BIN_PATH) directory..."
	cp -rf $(PKG_TMP_BIN_PATH)/$(PKG_NAME) $(PKG_BIN_PATH)/
	echo "Release size shrinking completed!"
endef

# Creates release files (tarballs, zipballs) 
define build_release_files =
	set -e
	set -u

	mkdir -p $(PKG_BIN_PATH)

	# Linux
	tar czvf $(PKG_BIN_PATH)/$(PKG_RELEASE_NAME).tar.gz -C $(PKG_TMP_BIN_PATH) $(PKG_NAME)
	sha256sum $(PKG_BIN_PATH)/$(PKG_RELEASE_NAME).tar.gz > $(PKG_BIN_PATH)/$(PKG_RELEASE_NAME)-SHA256SUM

	# Darwin
	tar czvf $(PKG_BIN_PATH)/$(PKG_RELEASE_NAME_DARWIN).tar.gz -C $(PKG_TMP_BIN_PATH_DARWIN) $(PKG_NAME)
	sha256sum $(PKG_BIN_PATH)/$(PKG_RELEASE_NAME_DARWIN).tar.gz > $(PKG_BIN_PATH)/$(PKG_RELEASE_NAME_DARWIN)-SHA256SUM

	du -sh $(PKG_BIN_PATH)/*
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
