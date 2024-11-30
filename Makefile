PKG_TARGET=x86_64-unknown-linux-musl
PKG_TARGET_DARWIN=x86_64-apple-darwin
RUST_VERSION ?= $(shell rustc --version | cut -d ' ' -f2)

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

lint:
	@rustc -vV
	@cargo clippy --all-features -- -D warnings
.PHONY: lint

build:
	@rustc -vV
	@cargo build --release --target $(PKG_TARGET)
.PHONY: build

# Release test tasks
test.release:
	@docker run --rm -it \
		-v $(PWD):/root/src/static-web-server \
		-v cargo-git:/root/.cargo/git \
		-v cargo-registry:/root/.cargo/registry \
		-v cargo-target:/root/src/static-web-server/target \
\
		--workdir /root/src/static-web-server \
		joseluisq/rust-linux-darwin-builder:$(RUST_VERSION) \
\
		bash -c "make prod.release"
.PHONY: test.release

linux:
	@docker run --rm -it \
		-v $(PWD):/root/src/static-web-server \
		-v cargo-git:/root/.cargo/git \
		-v cargo-registry:/root/.cargo/registry \
		-v cargo-target:/root/src/static-web-server/target \
\
		--workdir /root/src/static-web-server \
		joseluisq/rust-linux-darwin-builder:$(RUST_VERSION) \
\
		bash -c "\
			echo Building Linux release binary... && \
			rustc -vV && \
			cargo build --release --target $(PKG_TARGET) && \
			du -sh ./target/$(PKG_TARGET)/release/$(PKG_NAME) && \
			mkdir -p release && \
			cp -rf ./target/$(PKG_TARGET)/release/$(PKG_NAME) release/$(PKG_NAME)-linux && \
			echo \"Shrinking Linux binary file...\" && \
			strip release/$(PKG_NAME)-linux && \
			du -sh ./release/$(PKG_NAME)-linux"
.PHONY: linux

darwin:
	@docker run --rm -it \
		-v $(PWD):/root/src/static-web-server \
		-v cargo-git:/root/.cargo/git \
		-v cargo-registry:/root/.cargo/registry \
		-v cargo-target:/root/src/static-web-server/target \
\
		--workdir /root/src/static-web-server \
		joseluisq/rust-linux-darwin-builder:$(RUST_VERSION) \
\
		bash -c "\
			echo Building Darwin release binary... && \
			rustc -vV && \
			env CC=o64-clang CXX=o64-clang++ cargo build --release --target $(PKG_TARGET_DARWIN) && \
			du -sh ./target/$(PKG_TARGET_DARWIN)/release/$(PKG_NAME) && \
			mkdir -p release && \
			cp -rf ./target/$(PKG_TARGET_DARWIN)/release/$(PKG_NAME) release/$(PKG_NAME)-darwin && \
			echo \"Shrinking Darwin binary file...\" && \
			x86_64-apple-darwin20.2-strip release/$(PKG_NAME)-darwin && \
			du -sh ./release/$(PKG_NAME)-darwin"
.PHONY: darwin

#######################################
########### Utility tasks #############
#######################################

test:
	@echo "Testing application..."
	@rustc -vV
	@cargo test
.PHONY: test

fmt:
	@cargo fix
	@cargo fmt --all
.PHONY: fmt

## Development Docker images

docker.image:
	@echo "Creating development Docker Scratch image..."
	@cp -frp ./target/x86_64-unknown-linux-musl/release/static-web-server ./docker/devel/
	@docker build \
		--rm=true -f ./docker/devel/Dockerfile.scratch \
		--platform="linux/x86_64" \
		--network="host" \
		-t joseluisq/${PKG_NAME}:devel . --pull=true
.PHONY: docker.image

docker.image.alpine:
	@echo "Creating development Docker Alpine image..."
	@cp -frp ./target/x86_64-unknown-linux-musl/release/static-web-server ./docker/devel/
	@docker build \
		--rm=true -f ./docker/devel/Dockerfile.alpine \
		--platform="linux/x86_64" \
		--network="host" \
		-t joseluisq/${PKG_NAME}:devel-alpine . --pull=true
.PHONY: docker.image.alpine

docker.image.debian:
	@echo "Creating development Docker Alpine image..."
	@cp -frp ./target/x86_64-unknown-linux-musl/release/static-web-server ./docker/devel/
	@docker build \
		--platform="linux/x86_64" \
		--network="host" \
		--rm=true -f ./docker/devel/Dockerfile.debian \
		-t joseluisq/${PKG_NAME}:devel-debian . --pull=true
.PHONY: docker.image.debian

docker.image.all:
	@make docker.image
	@make docker.image.alpine
	@make docker.image.debian
.PHONY: docker.image.all


#######################################
########## Production tasks ###########
#######################################

# Compile release binary
define build_release =
	set -e
	set -u

	echo
	echo "Compiling application..."
	rustc -vV
	echo
	echo "Compiling release binary for $(PKG_TARGET)..."
	cargo build --release --target $(PKG_TARGET)
	echo
	echo
	echo "Compiling release binary for $(PKG_TARGET_DARWIN)..."
	env CC=o64-clang CXX=o64-clang++ cargo build --release --target $(PKG_TARGET_DARWIN)
	echo
	echo "Release builds were compiled!"
endef

# Shrink a release binary size
define build_release_shrink =
	set -e
	set -u

	echo "Copying release binaries..."

	mkdir -p $(PKG_BIN_PATH)

	# Linux
	mkdir -p $(PKG_TMP_BIN_PATH)
	cp -rf ./target/$(PKG_TARGET)/release/$(PKG_NAME) $(PKG_TMP_BIN_PATH)/

	# Darwin
	mkdir -p $(PKG_TMP_BIN_PATH_DARWIN)
	cp -rf ./target/$(PKG_TARGET_DARWIN)/release/$(PKG_NAME) $(PKG_TMP_BIN_PATH_DARWIN)/

	# Linux
	echo "Performing Linux/Darwin binaries shrinking..."
	echo "Binary sizes before:"
	du -sh $(PKG_TMP_BIN_PATH)/$(PKG_NAME)
	du -sh $(PKG_TMP_BIN_PATH_DARWIN)/$(PKG_NAME)

	# Shrink binaries in place (tmp dir)
	strip $(PKG_TMP_BIN_PATH)/$(PKG_NAME)
	x86_64-apple-darwin20.2-strip $(PKG_TMP_BIN_PATH_DARWIN)/$(PKG_NAME)

	echo "Binary sizes after (shrinking):"
	du -sh $(PKG_TMP_BIN_PATH)/$(PKG_NAME)
	du -sh $(PKG_TMP_BIN_PATH_DARWIN)/$(PKG_NAME)

	# Copy only Linux binary for the Docker image build process
	echo "Copying Linux binary from $(PKG_TMP_BIN_PATH) to $(PKG_BIN_PATH) directory..."
	cp -rf $(PKG_TMP_BIN_PATH)/$(PKG_NAME) $(PKG_BIN_PATH)/

	echo "Releases size shrinking completed!"
endef

# Creates release files (tarballs, zipballs)
define build_release_files =
	set -e
	set -u

	echo "Creating tarballs with their checksums..."

	# Enter to bin/ directory
	mkdir -p $(PKG_BIN_PATH)
	cd $(PKG_BIN_PATH)
	pwd

	# Tar/Gzip/sha256sum file for Linux
	tar czvf $(PKG_RELEASE_NAME).tar.gz -C $(PKG_TMP_BIN_PATH) $(PKG_NAME)
	sha256sum $(PKG_RELEASE_NAME).tar.gz > $(PKG_NAME)-v$(PKG_TAG)-SHA256SUM
	echo "Linux tarball with sha256sum created."

	# Tar/Gzip/sha256sum file for Darwin
	tar czvf $(PKG_RELEASE_NAME_DARWIN).tar.gz -C $(PKG_TMP_BIN_PATH_DARWIN) $(PKG_NAME)
	sha256sum $(PKG_RELEASE_NAME_DARWIN).tar.gz >> $(PKG_NAME)-v$(PKG_TAG)-SHA256SUM
	echo "Darwin tarball with sha256sum created."

	du -sh ./*
	echo "Release tarballs and sha256sum files created!"
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

prod.release.version:
	git add .
	git commit . -m "v$(PKG_TAG)"
	git tag v$(PKG_TAG)
	git push
.ONESHELL: prod.release.version

docs-dev:
	@docker-compose -f docs/docker-compose.yml up --build
.PHONY: docs-dev

crate-docs:
	@cargo doc --no-deps
.PHONY: crate-docs

crate-docs-dev:
	@env \
		RUSTDOCFLAGS="--cfg docsrs" \
			cargo doc --lib --no-deps --all-features --document-private-items
	@echo "Crate documentation: http://localhost:8787/static_web_server"
	@static-web-server -p 8787 -d target/doc/ \
		& watchman-make -p 'src/**/*.rs' --run '\
			env \
				RUSTDOCFLAGS="--cfg docsrs" \
					cargo doc --lib --no-deps --all-features --document-private-items'
.PHONY: crate-docs-dev

docs-deploy:
	@git stash
	@rm -rf /tmp/docs
	@mkdir -p /tmp/docs
	@docker-compose -f docs/docker-compose.yml build
	@docker run -it --rm \
		-v $(PWD)/.git:/docs/.git \
		-v $(PWD)/docs/content:/docs/docs/content \
		-v $(PWD)/docs/mkdocs.yml:/docs/mkdocs.yml \
		-v /tmp/docs:/tmp/docs \
			static-web-server-docs mkdocs build
	@git checkout gh-pages
	@git clean -fdx
	@rm -rf docs/
	@mkdir -p docs/
	@cp -rf /tmp/docs/. docs/
	@git add docs/
	@git commit docs/ -m "docs: automatic documentation updates [skip ci]"
	@git push origin gh-pages
	@git push github gh-pages
	@echo
	@echo "Documentation built and published"
	@git checkout master
.PHONY: docs-deploy

typos:
	@typos . --config ./.github/workflows/config/typos.toml
.PHONY: typos

man:
	@asciidoctor --doctype=manpage --backend=manpage docs/man/static-web-server.1.rst
.PHONY: man
