install:
	@cargo install --force cargo-make
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
