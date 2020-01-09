
all: run-basic run-debug run-optimize

build:
	cargo build

run-basic: build
	./target/debug/elm-torture $(ARGS) --suites suite --config ./.github/config/basic.json

run-debug: build
	./target/debug/elm-torture $(ARGS) --suites suite --config ./.github/config/debug.json

run-optimize: build
	./target/debug/elm-torture $(ARGS) --suites suite --config ./.github/config/optimize.json

.PHONEY: all build run-basic run-debug run-optimize
