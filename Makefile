BIN_NAME=static-web-server
PLATFORM=x86_64-unknown-linux-musl

start:
	-cargo run
.PHONY: start

watch:
	-cargo watch -cx run
.PHONY: watch

check:
	-cargo check
.PHONY: check

build:
	-cargo build
.PHONY: build

release:
	-cargo build --release --target $(PLATFORM)
	-mkdir -p bin
	-cp -rf target/$(PLATFORM)/release/${BIN_NAME} ./bin
	-strip ./bin/${BIN_NAME}
.PHONY: release

exec:
	./bin/${BIN_NAME}
.PHONY: exec

image:
	-docker build -t $(DOCKER_IMG) .
.PHONY: image

test:
	-echo "GET $(URL)" \
		| vegeta -cpus=12 attack \
			-workers=10 -duration=60s -connections=10000 -rate=200 -http2=false \
		| tee results.bin | vegeta report
	-cat results.bin | vegeta report -reporter=plot > plot.html
.PHONY: test
