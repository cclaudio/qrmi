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

pub mod fixtures;
pub mod mock_servers;

use rand::distr::{Alphanumeric, SampleString};

/// Initialize test environment (logging, etc.)
pub fn setup() {
    let _ = env_logger::builder().is_test(true).try_init();
}

/// Generate a random alphanumeric string of specified length
#[allow(dead_code)]
pub fn generate_random_string(len: usize) -> String {
    let mut rng = rand::rng();
    Alphanumeric.sample_string(&mut rng, len)
}

/// Generate a random UUID string
#[allow(dead_code)]
pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Set environment variables for a test backend
#[allow(dead_code)]
pub fn set_test_env_vars(backend_name: &str, vars: Vec<(&str, &str)>) {
    for (key, value) in vars {
        std::env::set_var(format!("{backend_name}_{key}"), value);
    }
}

/// Clear environment variables for a test backend
#[allow(dead_code)]
pub fn clear_test_env_vars(backend_name: &str, keys: Vec<&str>) {
    for key in keys {
        std::env::remove_var(format!("{backend_name}_{key}"));
    }
}

// Made with Bob
