# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

.PHONY: check-root check-rustfmt-installed check-clippy-installed check-python-version-installed check-venv-exists check-python-wheel-installed check-new-qrmi-version-valid get-qrmi-version

MAKEFLAGS += --no-print-directory

check-root:
	@if [ "$(shell id -u)" -ne 0 ]; then \
	  echo "Error: this target must be run as root (e.g. sudo make $(MAKECMDGOALS))"; \
	  exit 1; \
	fi

check-rustfmt-installed:
	@rustup component list --installed 2>/dev/null | grep -q '^rustfmt' || \
		{ echo "Error: rustfmt is not installed. Run: rustup component add rustfmt"; exit 1; }

check-clippy-installed:
	@rustup component list --installed 2>/dev/null | grep -q '^clippy' || \
		{ echo "Error: clippy is not installed. Run: rustup component add clippy"; exit 1; }

check-python-version-installed:
	@command -v python$(PYTHON_VERSION) >/dev/null 2>&1 || \
		{ echo "Error: PYTHON_VERSION variable is set to $(PYTHON_VERSION), but it is not installed in PATH"; exit 1; }

check-venv-exists:
	@if [ ! -d ".venv" ]; then
	  echo "Error: .venv not found. Run: make create-venv"; \
	  exit 1; \
	fi

check-python-wheel-installed: check-venv-exists
	@.venv/bin/pip show qrmi > /dev/null 2>&1 || \
	   (echo "qrmi wheel is NOT installed. Run: make install-python-wheel" && exit 1)

# Single source of truth to get the qrmi version.
get-qrmi-version:
	@grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/'

# Used to generate the correct wheel name (e.g. cp312) 
get-python-cp:
	@echo "cp$(PYTHON_VERSION) | sed 's/\.//'

# Before releasing a new qrmi version, we check if the current version was not released yet
check-new-qrmi-version-valid:
	@NEW_QRMI_VERSION=$$($(MAKE) get-qrmi-version); \
	if [ -z "$${NEW_QRMI_VERSION}" ]; then \
	  echo "Error: Failed to get QRMI version from Cargo.toml"; \
	  exit 1; \
	fi; \
	HTTP_CODE=$$(curl -s -o /dev/null -w "%{http_code}" -H "Accept: application/vnd.github+json" "https://api.github.com/repos/qiskit-community/qrmi/releases/tags/v$${NEW_QRMI_VERSION}"); \
	if [ "$${HTTP_CODE}" = "200" ]; then \
	  echo "Error: Release v$${NEW_QRMI_VERSION} already exists in GitHub releases"; \
	  echo "Please update the version in Cargo.toml and try again"; \
	  exit 1; \
	elif [ "$${HTTP_CODE}" != "404" ]; then \
	  echo "Error: Failed to check GitHub releases (HTTP $${HTTP_CODE})"; \
	  exit 1; \
	fi; \
	echo "Version v$${NEW_QRMI_VERSION} is valid and not yet released"

get-distro-platform-id:
	@. /etc/os-release 2>/dev/null && echo $${PLATFORM_ID:-$${ID}$${VERSION_ID}} | sed 's/platform://;s/:/-/g'

get-distro-packaging-type:
	@if command -v dnf >/dev/null 2>&1; then \
	  echo "rpm"; \
	elif command -v apt >/dev/null 2>&1; then \
	  echo "deb"; \
	else \
	  echo "Unknown packaging type"; \
	  exit 1; \
	fi
