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

//! C FFI Integration Tests
//!
//! These tests verify that the C API bindings work correctly with the Rust implementation.
//! They test the full workflow through the C interface to ensure proper memory management,
//! error handling, and data conversion between C and Rust.

use qrmi::cext::*;
use qrmi::models::{ResourceType, TaskStatus};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::common::{fixtures, mock_servers, generate_uuid};

/// Helper to convert Rust string to C string pointer
fn to_c_string(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

/// Helper to convert C string pointer to Rust string
unsafe fn from_c_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

/// Helper to setup environment variables for C FFI tests
fn setup_ffi_env_vars(backend: &str, da_endpoint: &str, iam_endpoint: &str, s3_endpoint: &str) {
    std::env::set_var(format!("{}_QRMI_IBM_DA_ENDPOINT", backend), da_endpoint);
    std::env::set_var(format!("{}_QRMI_IBM_DA_IAM_ENDPOINT", backend), iam_endpoint);
    std::env::set_var(format!("{}_QRMI_IBM_DA_IAM_APIKEY", backend), "test_api_key");
    std::env::set_var(format!("{}_QRMI_IBM_DA_SERVICE_CRN", backend), "crn:v1:test:service");
    std::env::set_var(format!("{}_QRMI_IBM_DA_AWS_ACCESS_KEY_ID", backend), "test_access_key");
    std::env::set_var(format!("{}_QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY", backend), "test_secret_key");
    std::env::set_var(format!("{}_QRMI_IBM_DA_S3_ENDPOINT", backend), s3_endpoint);
    std::env::set_var(format!("{}_QRMI_IBM_DA_S3_BUCKET", backend), "test-bucket");
    std::env::set_var(format!("{}_QRMI_IBM_DA_S3_REGION", backend), "us-east-1");
}

/// Helper to cleanup environment variables
fn cleanup_ffi_env_vars(backend: &str) {
    std::env::remove_var(format!("{}_QRMI_IBM_DA_ENDPOINT", backend));
    std::env::remove_var(format!("{}_QRMI_IBM_DA_IAM_ENDPOINT", backend));
    std::env::remove_var(format!("{}_QRMI_IBM_DA_IAM_APIKEY", backend));
    std::env::remove_var(format!("{}_QRMI_IBM_DA_SERVICE_CRN", backend));
    std::env::remove_var(format!("{}_QRMI_IBM_DA_AWS_ACCESS_KEY_ID", backend));
    std::env::remove_var(format!("{}_QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY", backend));
    std::env::remove_var(format!("{}_QRMI_IBM_DA_S3_ENDPOINT", backend));
    std::env::remove_var(format!("{}_QRMI_IBM_DA_S3_BUCKET", backend));
    std::env::remove_var(format!("{}_QRMI_IBM_DA_S3_REGION", backend));
}

/// Test: C FFI - String allocation and deallocation
#[test]
fn test_c_ffi_string_memory_management() {
    // Test string creation and freeing
    let test_str = "Hello from C FFI";
    let c_str = to_c_string(test_str);
    
    unsafe {
        // Verify string content
        let retrieved = from_c_string(c_str);
        assert_eq!(retrieved, test_str);
        
        // Free the string using C API
        let rc = qrmi_string_free(c_str);
        assert_eq!(rc as i32, ReturnCode::Success as i32);
    }
}

/// Test: C FFI - Null pointer handling
#[test]
fn test_c_ffi_null_pointer_handling() {
    unsafe {
        // Test null pointer to qrmi_string_free
        let rc = qrmi_string_free(ptr::null_mut());
        assert_eq!(rc as i32, ReturnCode::NullPointerError as i32);
    }
}

/// Test: C FFI - Resource creation and destruction
#[tokio::test]
async fn test_c_ffi_resource_lifecycle() {
    crate::common::setup();
    
    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;
    
    let backend = "test_c_ffi_backend";
    setup_ffi_env_vars(backend, &da_server.url(), &iam_server.url(), &s3_server.url());
    
    let _iam_mock = iam_server.mock_token_generation().await;
    let _backend_list_mock = da_server.mock_list_backends(backend).await;
    
    unsafe {
        // Create resource
        let backend_cstr = to_c_string(backend);
        let qrmi = qrmi_resource_new(backend_cstr, ResourceType::IBMDirectAccess);
        assert!(!qrmi.is_null(), "Resource creation should succeed");
        
        // Free resource
        let rc = qrmi_resource_free(qrmi);
        assert_eq!(rc as i32, ReturnCode::Success as i32);
        
        // Cleanup
        let _ = CString::from_raw(backend_cstr);
    }
    
    cleanup_ffi_env_vars(backend);
}

/// Test: C FFI - Resource accessibility check
#[tokio::test]
async fn test_c_ffi_is_accessible() {
    crate::common::setup();
    
    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;
    
    let backend = "test_c_ffi_accessible";
    setup_ffi_env_vars(backend, &da_server.url(), &iam_server.url(), &s3_server.url());
    
    let _iam_mock = iam_server.mock_token_generation().await;
    let _backend_list_mock = da_server.mock_list_backends(backend).await;
    let _backend_config_mock = da_server.mock_backend_configuration(backend).await;
    
    unsafe {
        let backend_cstr = to_c_string(backend);
        let qrmi = qrmi_resource_new(backend_cstr, ResourceType::IBMDirectAccess);
        assert!(!qrmi.is_null());
        
        // Check accessibility
        let mut accessible: bool = false;
        let rc = qrmi_resource_is_accessible(qrmi, &mut accessible as *mut bool);
        assert_eq!(rc as i32, ReturnCode::Success as i32);
        assert!(accessible, "Backend should be accessible");
        
        // Cleanup
        qrmi_resource_free(qrmi);
        let _ = CString::from_raw(backend_cstr);
    }
    
    cleanup_ffi_env_vars(backend);
}

/// Test: C FFI - Full workflow (acquire → task_start → task_status → release)
#[tokio::test]
async fn test_c_ffi_full_workflow() {
    crate::common::setup();
    
    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;
    
    let backend = "test_c_ffi_workflow";
    setup_ffi_env_vars(backend, &da_server.url(), &iam_server.url(), &s3_server.url());
    
    let _iam_mock = iam_server.mock_token_generation().await;
    let _backend_list_mock = da_server.mock_list_backends(backend).await;
    let _backend_config_mock = da_server.mock_backend_configuration(backend).await;
    
    let job_id = generate_uuid();
    let _job_submit_mock = da_server.mock_run_job(&job_id).await;
    let _status_completed_mock = da_server.mock_job_status_completed(&job_id).await;
    let _s3_put_mock = s3_server.mock_put_object("test-key").await;
    
    unsafe {
        let backend_cstr = to_c_string(backend);
        let qrmi = qrmi_resource_new(backend_cstr, ResourceType::IBMDirectAccess);
        assert!(!qrmi.is_null());
        
        // Acquire resource
        let mut acquisition_token: *mut c_char = ptr::null_mut();
        let rc = qrmi_resource_acquire(qrmi, &mut acquisition_token as *mut *mut c_char);
        assert_eq!(rc as i32, ReturnCode::Success as i32);
        assert!(!acquisition_token.is_null());
        
        let token_str = from_c_string(acquisition_token);
        assert!(!token_str.is_empty());
        
        // Start task
        let input_cstr = to_c_string(&fixtures::estimator_v2_input());
        let program_id_cstr = to_c_string("estimator");
        
        let payload = Payload::QiskitPrimitive {
            input: input_cstr,
            program_id: program_id_cstr,
        };
        
        let mut task_id: *mut c_char = ptr::null_mut();
        let rc = qrmi_resource_task_start(qrmi, &payload as *const Payload, &mut task_id as *mut *mut c_char);
        assert_eq!(rc as i32, ReturnCode::Success as i32);
        assert!(!task_id.is_null());
        
        let task_id_str = from_c_string(task_id);
        assert!(!task_id_str.is_empty());
        
        // Check task status
        let mut status = TaskStatus::Queued;
        let rc = qrmi_resource_task_status(qrmi, task_id, &mut status as *mut TaskStatus);
        assert_eq!(rc as i32, ReturnCode::Success as i32);
        
        // Stop task
        let rc = qrmi_resource_task_stop(qrmi, task_id);
        assert_eq!(rc as i32, ReturnCode::Success as i32);
        
        // Release resource
        let rc = qrmi_resource_release(qrmi, acquisition_token);
        assert_eq!(rc as i32, ReturnCode::Success as i32);
        
        // Cleanup
        qrmi_string_free(acquisition_token);
        qrmi_string_free(task_id);
        qrmi_resource_free(qrmi);
        let _ = CString::from_raw(backend_cstr);
        let _ = CString::from_raw(input_cstr);
        let _ = CString::from_raw(program_id_cstr);
    }
    
    cleanup_ffi_env_vars(backend);
}

/// Test: C FFI - Target retrieval
#[tokio::test]
async fn test_c_ffi_target_retrieval() {
    crate::common::setup();
    
    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;
    
    let backend = "test_c_ffi_target";
    setup_ffi_env_vars(backend, &da_server.url(), &iam_server.url(), &s3_server.url());
    
    let _iam_mock = iam_server.mock_token_generation().await;
    let _backend_config_mock = da_server.mock_backend_configuration(backend).await;
    let _backend_props_mock = da_server.mock_backend_properties(backend).await;
    
    unsafe {
        let backend_cstr = to_c_string(backend);
        let qrmi = qrmi_resource_new(backend_cstr, ResourceType::IBMDirectAccess);
        assert!(!qrmi.is_null());
        
        // Get target
        let mut target: *mut c_char = ptr::null_mut();
        let rc = qrmi_resource_target(qrmi, &mut target as *mut *mut c_char);
        assert_eq!(rc as i32, ReturnCode::Success as i32);
        assert!(!target.is_null());
        
        let target_str = from_c_string(target);
        assert!(!target_str.is_empty());
        assert!(target_str.contains("configuration") || target_str.contains("backend"));
        
        // Cleanup
        qrmi_string_free(target);
        qrmi_resource_free(qrmi);
        let _ = CString::from_raw(backend_cstr);
    }
    
    cleanup_ffi_env_vars(backend);
}

/// Test: C FFI - Metadata retrieval
#[tokio::test]
async fn test_c_ffi_metadata_retrieval() {
    crate::common::setup();
    
    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;
    
    let backend = "test_c_ffi_metadata";
    setup_ffi_env_vars(backend, &da_server.url(), &iam_server.url(), &s3_server.url());
    
    let _iam_mock = iam_server.mock_token_generation().await;
    
    unsafe {
        let backend_cstr = to_c_string(backend);
        let qrmi = qrmi_resource_new(backend_cstr, ResourceType::IBMDirectAccess);
        assert!(!qrmi.is_null());
        
        // Get metadata
        let mut metadata: *mut ResourceMetadata = ptr::null_mut();
        let rc = qrmi_resource_metadata(qrmi, &mut metadata as *mut *mut ResourceMetadata);
        assert_eq!(rc as i32, ReturnCode::Success as i32);
        assert!(!metadata.is_null());
        
        // Get metadata keys
        let mut num_keys: usize = 0;
        let mut key_names: *mut *mut c_char = ptr::null_mut();
        let rc = qrmi_resource_metadata_keys(
            metadata,
            &mut num_keys as *mut usize,
            &mut key_names as *mut *mut *mut c_char
        );
        assert_eq!(rc as i32, ReturnCode::Success as i32);
        assert!(num_keys > 0, "Should have at least one metadata key");
        
        // Cleanup
        if !key_names.is_null() {
            qrmi_string_array_free(num_keys, key_names);
        }
        qrmi_resource_metadata_free(metadata);
        qrmi_resource_free(qrmi);
        let _ = CString::from_raw(backend_cstr);
    }
    
    cleanup_ffi_env_vars(backend);
}

/// Test: C FFI - Error handling with invalid resource
#[test]
fn test_c_ffi_error_handling() {
    unsafe {
        // Try to use null resource pointer
        let mut accessible: bool = false;
        let rc = qrmi_resource_is_accessible(ptr::null_mut(), &mut accessible as *mut bool);
        assert_eq!(rc as i32, ReturnCode::NullPointerError as i32);
        
        // Try to acquire with null resource
        let mut token: *mut c_char = ptr::null_mut();
        let rc = qrmi_resource_acquire(ptr::null_mut(), &mut token as *mut *mut c_char);
        assert_eq!(rc as i32, ReturnCode::NullPointerError as i32);
    }
}

/// Test: C FFI - Last error retrieval
#[test]
fn test_c_ffi_last_error() {
    unsafe {
        // Trigger an error
        let rc = qrmi_resource_free(ptr::null_mut());
        assert_eq!(rc as i32, ReturnCode::NullPointerError as i32);
        
        // Get last error
        let error_ptr = qrmi_get_last_error();
        assert!(!error_ptr.is_null());
        
        let error_msg = from_c_string(error_ptr);
        assert!(!error_msg.is_empty());
    }
}

/// Test: C FFI - Payload memory management
#[test]
fn test_c_ffi_payload_memory() {
    let input = to_c_string(&fixtures::estimator_v2_input());
    let program_id = to_c_string("estimator");
    
    let payload = Payload::QiskitPrimitive {
        input,
        program_id,
    };
    
    // Verify payload structure
    match payload {
        Payload::QiskitPrimitive { input: i, program_id: p } => {
            unsafe {
                assert!(!i.is_null());
                assert!(!p.is_null());
                
                let input_str = from_c_string(i);
                let program_str = from_c_string(p);
                
                assert!(!input_str.is_empty());
                assert_eq!(program_str, "estimator");
                
                // Cleanup
                let _ = CString::from_raw(i);
                let _ = CString::from_raw(p);
            }
        },
        _ => panic!("Wrong payload type"),
    }
}

/// Test: C FFI - Multiple resources simultaneously
#[tokio::test]
async fn test_c_ffi_multiple_resources() {
    crate::common::setup();
    
    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;
    
    let backend1 = "test_c_ffi_multi_1";
    let backend2 = "test_c_ffi_multi_2";
    
    setup_ffi_env_vars(backend1, &da_server.url(), &iam_server.url(), &s3_server.url());
    setup_ffi_env_vars(backend2, &da_server.url(), &iam_server.url(), &s3_server.url());
    
    let _iam_mock = iam_server.mock_token_generation().await;
    let _backend1_list = da_server.mock_list_backends(backend1).await;
    let _backend2_list = da_server.mock_list_backends(backend2).await;
    
    unsafe {
        // Create two resources
        let backend1_cstr = to_c_string(backend1);
        let backend2_cstr = to_c_string(backend2);
        
        let qrmi1 = qrmi_resource_new(backend1_cstr, ResourceType::IBMDirectAccess);
        let qrmi2 = qrmi_resource_new(backend2_cstr, ResourceType::IBMDirectAccess);
        
        assert!(!qrmi1.is_null());
        assert!(!qrmi2.is_null());
        assert_ne!(qrmi1, qrmi2, "Resources should be different");
        
        // Cleanup
        qrmi_resource_free(qrmi1);
        qrmi_resource_free(qrmi2);
        let _ = CString::from_raw(backend1_cstr);
        let _ = CString::from_raw(backend2_cstr);
    }
    
    cleanup_ffi_env_vars(backend1);
    cleanup_ffi_env_vars(backend2);
}

// Made with Bob
