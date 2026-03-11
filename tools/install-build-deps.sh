#!/bin/bash

# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

# This script installs the build dependencies requided to run the Makefile

packages="\
	gcc \
	make \
"

if [ "$(id -u)" -ne 0 ]; then
	echo "Error: root privilege required. Run sudo $0"
	exit 1
fi

if command -v dnf >/dev/null 2>&1; then
	dnf install -y $packages
elif command -v apt >/dev/null 2>&1; then
	apt update && apt install -y $packages
else
	echo "Distro not supported"
	exit 1
fi
