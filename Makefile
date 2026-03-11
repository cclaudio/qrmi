# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

DIST_DIR ?= .
PYTHON_VERSION ?= 3.12

.PHONY: all build build-rust-all build-rust-with-examples fmt-rust-all lint-rust-all lint-rust-with-examples
.PHONY: test-rust-all test-rust-with-examples test-rust-bins test-rust-dependencies
.PHONY: build-c-examples clean-c-examples
.PHONY: create-venv build-python-wheel fmt-python-wheel install-python-wheel lint-python-wheel test-python-wheel
.PHONY: source-tarball vendor-tarball libqrmi-tarball clean-tarballs
.PHONY: clean clean-all help

all: build

include Makefile_helpers.mk

QRMI_VERSION = $$($(MAKE) get-qrmi-version)
PYTHON_CP = $$($(MAKE) get-python-cp)

# ------------------------------------------------
# Rust targets
# ------------------------------------------------

build:
	cargo build --locked --release

build-rust-all: build-rust-with-examples
	cargo build --locked --bin task_runner --release --features="build-binary"
	cargo build --locked --bin stubgen --release --features="pyo3"

build-rust-with-examples:
	cargo build --all-targets --release

# Format check libqrmi, dependencies, examples and binaries
fmt-rust-all: check-rustfmt-installed
	cargo fmt --all -- --check --verbose

lint-rust-all: lint-with-examples
	cargo clippy --locked --bin task_runner --features="build-binary" -- -D warnings
	cargo clippy --locked --bin stubgen --features="pyo3" -- -D warnings

lint-rust-with-examples: check-clippy-installed
	cargo clippy --locked --all-targets -- -D warnings

test-rust-all: test-rust-with-examples test-rust-bins test-rust-dependencies

test-rust-with-examples:
	cargo test --locked --all-targets --release

test-rust-bins:
	cargo test --locked --bin task_runner --release --features="build-binary"
	cargo test --locked --bin stubgen --release --features="pyo3"

test-rust-dependencies:
	cargo test --locked --release -p direct-access-api
	cargo test --locked --release -p pasqal-cloud-api
	cargo test --locked --release -p qiskit_runtime_client

# ------------------------------------------------
# C examples targets
# ------------------------------------------------

build-c-examples: build
	@mkdir -p examples/qrmi/c/direct_access/build
	@cd examples/qrmi/c/direct_access/build && \
		cmake -DCMAKE_BUILD_TYPE=Release .. && \
		cmake --build .
	@mkdir -p examples/qrmi/c/qiskit_runtime_service/build
	@cd examples/qrmi/c/qiskit_runtime_service/build && \
		cmake -DCMAKE_BUILD_TYPE=Release .. && \
		cmake --build .
	@mkdir -p examples/qrmi/c/pasqal_cloud/build
	@cd examples/qrmi/c/pasqal_cloud/build && \
		cmake -DCMAKE_BUILD_TYPE=Release .. && \
		cmake --build .
	@mkdir -p examples/qrmi/c/config/build
	@cd examples/qrmi/c/config/build && \
		cmake -DCMAKE_BUILD_TYPE=Release .. && \
		cmake --build .

clean-c-examples:
	@rm -rf examples/qrmi/c/direct_access/build
	@rm -rf examples/qrmi/c/qiskit_runtime_service/build
	@rm -rf examples/qrmi/c/pasqal_cloud/build
	@rm -rf examples/qrmi/c/config/build

# ------------------------------------------------
# Python targets
# ------------------------------------------------

create-venv: check-python-version-installed
	@if [ -d ".venv" ]; then \
	  echo "Error: .venv already exists. Remove it first or just skip this step."; \
	  exit 1; \
	fi
	python$(PYTHON_VERSION) -m venv .venv
	.venv/bin/pip install --upgrade pip
	.venv/bin/pip install -r requirements-dev.txt
	@echo "Virtual environment created. You can activate it with: source .venv/bin/activate"

build-python-wheel: check-venv-exists
	maturin build --release

install-python-wheel: check-venv-exists
	@if ! ls target/wheels/qrmi-$(QRMI_VERSION)-$(PYTHON_CP)-*.whl 1> /dev/null 2>&1; then \
	  echo "target/wheels/qrmi-$(QRMI_VERSION)-$(PYTHON_CP)-*.whl not found. Run make build-python-wheel"; \
	  exit 1; \
	fi
	.venv/bin/pip install --force-reinstall target/wheels/qrmi-$(PYTHON_VERSION)-$(PYTHON_CP)-*.whl

fmt-python-wheel: check-venv-exists
	.venv/bin/black --check ./python

lint-python-wheel: check-venv-exists check-python-wheel-installed
	.venv/bin/pylint ./python

test-python-wheel: check-venv-exists check-python-wheel-installed
	.venv/bin/pytest python/tests/

# ------------------------------------------------
# Packaging targets
# ------------------------------------------------

tarball-source:
	@git archive --format=tar.gz \
	  --prefix=qrmi-$(QRMI_VERSION)/ \
	  HEAD \
	  -o $(DIST_DIR)/qrmi-$(QRMI_VERSION).tar.gz
	@sha256sum $(DIST_DIR)/qrmi-$(QRMI_VERSION).tar.gz > $(DIST_DIR)/qrmi-$(QRMI_VERSION).tar.gz.sha256
	@echo "Created: $(DIST_DIR)/qrmi-$(QRMI_VERSION).tar.gz"
	@echo "SHA256: $(DIST_DIR)/qrmi-$(QRMI_VERSION).tar.gz.sha256"

# Allow disconnected/airgapped build
tarball-vendor:
	cargo vendor $(DIST_DIR)/vendor
	tar czf $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz -C $(DIST_DIR) vendor/
	rm -rf $(DIST_DIR)/vendor
	@sha256sum $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz > $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz.sha256
	@echo "Created: $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz"
	@echo "SHA256: $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz.sha256"

tarball-libqrmi: build
	@TARFILES="target/release/libqrmi.so qrmi.h LICENSE.txt"; \
	PLATFORM_ID=$$($(MAKE) get-distro-platform-id); \
	TARBALL="$(DIST_DIR)/libqrmi-$(QRMI_VERSION)-$${PLATFORM_ID}-$$(arch).tar.gz"; \
	WORKDIR="$(DIST_DIR)/libqrmi-$(QRMI_VERSION)-$${PLATFORM_ID}-$$(arch)"; \
	mkdir -p $${WORKDIR}; \
	cp $${TARFILES} $${WORKDIR}; \
	tar czf $${TARBALL} -C $$(dirname $${WORKDIR}) $$(basename $${WORKDIR})"; \
	rm -rf $${WORKDIR}; \
	echo "Created: $${TARBALL}";

tarball-qrmi-wheel: build-python-wheel
	@TARFILES="target/wheels/qrmi-$(QRMI_VERSION)-$(PYTHON_CP)-*.whl LICENSE.txt"; \
	PLATFORM_ID=$$($(MAKE) get-distro-platform-id); \
	TARBALL="$(DIST_DIR)/qrmi-wheel-$(QRMI_VERSION)-$(PYTHON_CP)-$${PLATFORM_ID}-$$(arch).tar.gz"; \
	WORKDIR="$(DIST_DIR)/qrmi-wheel-$(QRMI_VERSION)-$(PYTHON_CP)-$${PLATFORM_ID}-$$(arch)"; \
	mkdir -p $${WORKDIR}; \
	cp $${TARFILES} $${WORKDIR}; \
	tar czf $${TARBALL} -C $$(dirname $${WORKDIR}) $$(basename $${WORKDIR})"; \
	rm -rf $${WORKDIR}; \
	echo "Created: $${TARBALL}";

clean-tarballs:
	rm -f $(DIST_DIR)/libqrmi-$(QRMI_VERSION)-*.tar.gz
	rm -f $(DIST_DIR)/qrmi-wheel-$(QRMI_VERSION)-*.tar.gz

# ------------------------------------------------
# Misc targets
# ------------------------------------------------

clean:
	cargo clean

clean-all: clean clean-c-examples clean-tarballs

help:
	@echo "Usage: make [TARGET]"
	@echo
	@echo "Rust targets:"
	@echo
	@echo "  all                      - Same as \"make build\" (default)"
	@echo "  build                    - Build libqrmi.so and qrmi.h"
	@echo "  build-rust-with-examples - Build libqrmi with --all-targets (include examples, but not binaries)"
	@echo "  build-rust-all           - Build libqrmi with --all-targets, and also build binaries"
	@echo "  fmt-rust-all             - Format check all Rust code (libqrmi, examples and binaries)"
	@echo "  lint-rust-with-examples  - Run cargo clippy on libqrmi with --all-targets, which includes examples, but not binaries"
	@echo "  lint-rust-all            - Run clippy on libqrmi all Rust code (libqrmi, dependencies, examples, and binaries)"
	@echo "  test-rust-with-examples  - Run tests on libqrmi with --all-targets"
	@echo "  test-rust-bins           - Run tests on binaries (task_runner and stubgen)"
	@echo "  test-rust-dependencies   - Run tests on dependency crates"
	@echo "  test-rust-all            - Run all Rust tests (with-examples, bins, and dependencies)"
	@echo
	@echo "C targets:"
	@echo
	@echo "  build-c-examples - Build all C examples from the examples/qrmi/c/ directory"
	@echo "  clean-c-examples - Remove C examples built previously"
	@echo
	@echo "Python targets:"
	@echo
	@echo "  create-venv          - Create a Python virtual environment in .venv"
	@echo "  build-python-wheel   - Build a Python wheel for qrmi using maturin"
	@echo "  fmt-python-wheel     - Format check Python code using black"
	@echo "  install-python-wheel - Install the qrmi Python wheel in .venv"
	@echo "  lint-python-wheel    - Lint Python code using pylint"
	@echo "  test-python-wheel    - Run Python tests using pytest"
	@echo
	@echo "Packaging targets:"
	@echo
	@echo "  The following targets are used to create tarballs for new releases. The tarballs are created in DIST_DIR (default: ./) from git HEAD."
	@echo "  Before creating a new release, we encourage running 'make check-new-qrmi-version-valid'"
	@echo
	@echo "    source-tarball      - Create the qrmi-<QRMI_VERSION>.tar.gz in DIST_DIR"
	@echo "    vendor-tarball      - Create the qrmi-<QRMI_VERSION>-vendor.tar.gz in DIST_DIR with vendor Rust dependencies"
	@echo "    libqrmi-tarball     - Create the libqrmi-<QRMI_VERSION>-el8-x86_64.tar.gz in DIST_DIR with libqrmi.so and qrmi.h"
	@echo "    clean-tarballs      - Remove all generated tarballs"
	@echo
	@echo "Common targets:"
	@echo
	@echo "  clean     - Remove the target directory"
	@echo "  clean-all - Remove target directory, C examples builds, and tarballs"
	@echo "  help      - Show this help message"
	@echo
	