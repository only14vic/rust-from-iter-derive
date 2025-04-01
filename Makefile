-include .env
export

SHELL = sh
.DEFAULT_GOAL = help

ifndef VERBOSE
.SILENT:
endif

make = make --no-print-directory

expand:
	cargo expand --test test

test:
	cargo test $(args) -- --nocapture --color always
