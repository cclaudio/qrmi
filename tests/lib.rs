// This code is part of Qiskit.
//
// (C) Copyright IBM 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Integration tests for QRMI (Quantum Resource Management Interface)
//!
//! This test suite provides comprehensive end-to-end testing for all quantum providers:
//! - IBM Direct Access
//! - IBM Qiskit Runtime Service
//! - Pasqal Cloud
//!
//! Each provider test suite covers:
//! - Full workflow: acquire → task_start → task_status → task_result → release
//! - Authentication scenarios (success and failure)
//! - Job execution scenarios (success, failure, cancellation)
//! - Resource accessibility checks
//! - Target and metadata retrieval
//!
//! ## Running Tests
//!
//! Run all integration tests:
//! ```bash
//! cargo test --test lib
//! ```
//!
//! Run tests for a specific provider:
//! ```bash
//! cargo test --test lib ibm_direct_access
//! cargo test --test lib ibm_qrs
//! cargo test --test lib pasqal_cloud
//! ```
//!
//! Run a specific test:
//! ```bash
//! cargo test --test lib test_full_workflow_estimator_success
//! ```

mod common;
mod integration;

// Made with Bob
