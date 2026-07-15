# This code is part of Qiskit.
#
# Copyright (C) IBM 2026
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Example showing how to retrieve the QRMI API version."""

from qrmi._core import (
    QRMI_API_VERSION_MAJOR,
    QRMI_API_VERSION_MINOR,
    QRMI_API_VERSION_PATCH)

print(f"{QRMI_API_VERSION_MAJOR}.{QRMI_API_VERSION_MINOR}.{QRMI_API_VERSION_PATCH}")
