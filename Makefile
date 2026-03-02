# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

VERSION := $(shell grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
DIST_DIR ?= .

include Makefile.helpers

.PHONY: all build dist dist-lib clean help

all: build

# ------------------------------------------------
# Rust targets
# ------------------------------------------------

build:
	cargo build --locked --release

# ------------------------------------------------
# Packaging targets
# ------------------------------------------------

dist:
	git archive --format=tar.gz \
	  --prefix=qrmi-$(VERSION)/ \
	  HEAD \
	  -o $(DIST_DIR)/qrmi-$(VERSION).tar.gz
	cargo vendor $(DIST_DIR)/vendor
	tar czf $(DIST_DIR)/qrmi-$(VERSION)-vendor.tar.gz vendor/
	rm -rf $(DIST_DIR)/vendor

dist-rhel-lib: build
	ARCH=$$(uname -m); \
	TARBALL="libqrmi-$(VERSION)-rhel8-x86_64.tar.gz"; \
	mkdir -p $(DIST_DIR)/libqrmi-$(VERSION); \
	cp target/release/libqrmi.so $(DIST_DIR)/libqrmi-$(VERSION)/; \
	cp qrmi.h $(DIST_DIR)/libqrmi-$(VERSION)/; \
	cp LICENSE.txt $(DIST_DIR)/libqrmi-$(VERSION)/; \
	tar czf $(DIST_DIR)/$${TARBALL} -C $(DIST_DIR) libqrmi-$(VERSION); \
	rm -rf $(DIST_DIR)/libqrmi-$(VERSION); \
	echo "Created: $(DIST_DIR)/$${TARBALL}"

install-rhel-build-deps: check-root
	dnf install -y git gcc make

# ------------------------------------------------
# Misc targets
# ------------------------------------------------

clean:
	cargo clean

help:
	@echo "Rust targets:"
	@echo
	@echo "  all    - same as \"make build\" (default)"
	@echo "  build  - Build libqrmi"
	@echo
	@echo "Packaging targets:"
	@echo
	@echo "  dist       - Create source and vendor tarballs in DIST_DIR (default: ./)"
	@echo "  dist-lib   - Create libqrmi tarball with libqrmi.so and qrmi.h"
	@echo "  install-rhel-build-deps - Install package build dependencies"
	@echo
	@echo "Misc targets:"
	@echo
	@echo "  clean  - Remove build artifacts"
	@echo "  help   - Show this help message"
	