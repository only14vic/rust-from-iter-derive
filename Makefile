-include .env
export

SHELL = sh
.DEFAULT_GOAL = help

ifndef VERBOSE
.SILENT:
endif

make = make --no-print-directory

perf:
	cargo build --tests --release --no-default-features
	perf record -F99 --call-graph dwarf \
		"$(shell find target -type f -executable -path */release/deps/test-*)"
	perf report

expand:
	cargo expand --test test

test:
	cargo test $(args) -- --nocapture --color always
