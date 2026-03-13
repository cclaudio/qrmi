#!/bin/bash

# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

# This script installs the Makefile bootstrap packages, they must be
# installed before running make.

# List of packages to be installed
BOOTSTRAP_PACKAGES="\
  gcc \
  make \
  cmake \
"

# Check root privilege
if [ "$(id -u)" -ne 0 ]; then
  echo "Error: root privilege required. Run sudo $0"
  exit 1
fi

if command -v dnf >/dev/null 2>&1; then
  dnf install -y ${BOOTSTRAP_PACKAGES}
else
  echo "Linux distribution not supported"
  exit 1
fi

