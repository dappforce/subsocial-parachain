CARGO=cargo

.PHONY: init
init: 
	./scripts/init.sh

.PHONY: build
build:
	${CARGO} build --release
build-runtime:
	./scripts/build-runtime.sh

.PHONY: compare
compare-ordering:
	./scripts/manual-compare-ordering.sh

.PHONY: benchmark
benchmark: 
	./scripts/run-benchmark-on.sh