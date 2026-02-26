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

use qrmi::ibm::IBMDirectAccess;
use qrmi::models::{Payload, TaskStatus};
use qrmi::QuantumResource;

use crate::common::{fixtures, mock_servers, set_test_env_vars, clear_test_env_vars, generate_uuid};

const TEST_BACKEND: &str = "test_ibm_da_backend";

/// Helper to setup environment variables for IBM Direct Access tests
fn setup_da_env_vars(
    da_endpoint: &str,
    iam_endpoint: &str,
    s3_endpoint: &str,
) {
    set_test_env_vars(
        TEST_BACKEND,
        vec![
            ("QRMI_IBM_DA_ENDPOINT", da_endpoint),
            ("QRMI_IBM_DA_IAM_ENDPOINT", iam_endpoint),
            ("QRMI_IBM_DA_IAM_APIKEY", "test_api_key"),
            ("QRMI_IBM_DA_SERVICE_CRN", "crn:v1:test:service"),
            ("QRMI_IBM_DA_AWS_ACCESS_KEY_ID", "test_access_key"),
            ("QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY", "test_secret_key"),
            ("QRMI_IBM_DA_S3_ENDPOINT", s3_endpoint),
            ("QRMI_IBM_DA_S3_BUCKET", "test-bucket"),
            ("QRMI_IBM_DA_S3_REGION", "us-east-1"),
        ],
    );
}

/// Helper to cleanup environment variables
fn cleanup_da_env_vars() {
    clear_test_env_vars(
        TEST_BACKEND,
        vec![
            "QRMI_IBM_DA_ENDPOINT",
            "QRMI_IBM_DA_IAM_ENDPOINT",
            "QRMI_IBM_DA_IAM_APIKEY",
            "QRMI_IBM_DA_SERVICE_CRN",
            "QRMI_IBM_DA_AWS_ACCESS_KEY_ID",
            "QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY",
            "QRMI_IBM_DA_S3_ENDPOINT",
            "QRMI_IBM_DA_S3_BUCKET",
            "QRMI_IBM_DA_S3_REGION",
        ],
    );
}

/// Test: Full workflow - Happy path with Estimator
/// Tests: acquire → task_start → task_status → task_result → release
#[tokio::test]
async fn test_full_workflow_estimator_success() {
    common::setup();

    // Setup mock servers
    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;

    // Setup environment
    setup_da_env_vars(&da_server.url(), &iam_server.url(), &s3_server.url());

    // Mock IAM token generation
    let _iam_mock = iam_server.mock_token_generation().await;

    // Mock backend operations
    let _backend_list_mock = da_server.mock_list_backends(TEST_BACKEND).await;
    let _backend_config_mock = da_server.mock_backend_configuration(TEST_BACKEND).await;

    // Generate job ID
    let job_id = generate_uuid();

    // Mock job submission
    let _job_submit_mock = da_server.mock_run_job(&job_id).await;

    // Mock job status transitions: Queued → Running → Completed
    let _status_queued_mock = da_server.mock_job_status_queued(&job_id).await;
    let _status_running_mock = da_server.mock_job_status_running(&job_id).await;
    let _status_completed_mock = da_server.mock_job_status_completed(&job_id).await;

    // Mock job details
    let _job_details_mock = da_server.mock_job_details(&job_id, "Completed").await;

    // Mock S3 operations
    let _s3_put_mock = s3_server.mock_put_object("test-key").await;
    let _s3_get_mock = s3_server
        .mock_get_object("test-key", &fixtures::job_result_success().to_string())
        .await;

    // Create QRMI instance
    let mut qrmi = IBMDirectAccess::new(TEST_BACKEND).expect("Failed to create QRMI instance");

    // Test: is_accessible
    let accessible = qrmi
        .is_accessible()
        .await
        .expect("Failed to check accessibility");
    assert!(accessible, "Backend should be accessible");

    // Test: acquire
    let acquisition_token = qrmi.acquire().await.expect("Failed to acquire resource");
    assert!(!acquisition_token.is_empty(), "Acquisition token should not be empty");

    // Test: task_start with Estimator payload
    let payload = Payload::QiskitPrimitive {
        input: fixtures::estimator_v2_input(),
        program_id: "estimator".to_string(),
    };

    let task_id = qrmi
        .task_start(payload)
        .await
        .expect("Failed to start task");
    assert!(!task_id.is_empty(), "Task ID should not be empty");

    // Test: task_status - should eventually be Completed
    // In real scenario, we'd poll; here we just check final state
    let status = qrmi
        .task_status(&task_id)
        .await
        .expect("Failed to get task status");
    assert!(
        matches!(status, TaskStatus::Queued | TaskStatus::Running | TaskStatus::Completed),
        "Status should be valid"
    );

    // Test: task_result
    let result = qrmi
        .task_result(&task_id)
        .await
        .expect("Failed to get task result");
    assert!(!result.value.is_empty(), "Result should not be empty");

    // Test: task_stop
    qrmi.task_stop(&task_id)
        .await
        .expect("Failed to stop task");

    // Test: release
    qrmi.release(&acquisition_token)
        .await
        .expect("Failed to release resource");

    // Cleanup
    cleanup_da_env_vars();
}

/// Test: Full workflow with Sampler
#[tokio::test]
async fn test_full_workflow_sampler_success() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;

    setup_da_env_vars(&da_server.url(), &iam_server.url(), &s3_server.url());

    let _iam_mock = iam_server.mock_token_generation().await;
    let _backend_list_mock = da_server.mock_list_backends(TEST_BACKEND).await;
    let _backend_config_mock = da_server.mock_backend_configuration(TEST_BACKEND).await;

    let job_id = generate_uuid();
    let _job_submit_mock = da_server.mock_run_job(&job_id).await;
    let _status_completed_mock = da_server.mock_job_status_completed(&job_id).await;
    let _job_details_mock = da_server.mock_job_details(&job_id, "Completed").await;

    let _s3_put_mock = s3_server.mock_put_object("test-key").await;
    let _s3_get_mock = s3_server
        .mock_get_object("test-key", &fixtures::job_result_success().to_string())
        .await;

    let mut qrmi = IBMDirectAccess::new(TEST_BACKEND).expect("Failed to create QRMI instance");

    let _token = qrmi.acquire().await.expect("Failed to acquire");

    // Test with Sampler payload
    let payload = Payload::QiskitPrimitive {
        input: fixtures::sampler_v2_input(),
        program_id: "sampler".to_string(),
    };

    let task_id = qrmi.task_start(payload).await.expect("Failed to start task");
    assert!(!task_id.is_empty());

    let status = qrmi.task_status(&task_id).await.expect("Failed to get status");
    assert!(matches!(status, TaskStatus::Completed | TaskStatus::Queued | TaskStatus::Running));

    cleanup_da_env_vars();
}

/// Test: Authentication failure
#[tokio::test]
async fn test_authentication_failure() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;

    setup_da_env_vars(&da_server.url(), &iam_server.url(), &s3_server.url());

    // Mock IAM token generation failure
    let _iam_mock = iam_server.mock_token_generation_failure().await;

    // Attempting to create QRMI should work, but operations should fail
    let mut qrmi = IBMDirectAccess::new(TEST_BACKEND).expect("Failed to create QRMI instance");

    // Operations requiring authentication should fail
    let result = qrmi.is_accessible().await;
    assert!(result.is_err(), "Should fail with authentication error");

    cleanup_da_env_vars();
}

/// Test: Backend not accessible
#[tokio::test]
async fn test_backend_not_accessible() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;

    setup_da_env_vars(&da_server.url(), &iam_server.url(), &s3_server.url());

    let _iam_mock = iam_server.mock_token_generation().await;
    
    // Mock backend not found
    let _backend_not_found_mock = da_server.mock_backend_not_found(TEST_BACKEND).await;

    let mut qrmi = IBMDirectAccess::new(TEST_BACKEND).expect("Failed to create QRMI instance");

    let result = qrmi.is_accessible().await;
    // Should either return false or error depending on implementation
    assert!(result.is_err() || !result.unwrap(), "Backend should not be accessible");

    cleanup_da_env_vars();
}

/// Test: Job execution failure
#[tokio::test]
async fn test_job_execution_failure() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;

    setup_da_env_vars(&da_server.url(), &iam_server.url(), &s3_server.url());

    let _iam_mock = iam_server.mock_token_generation().await;
    let _backend_list_mock = da_server.mock_list_backends(TEST_BACKEND).await;
    let _backend_config_mock = da_server.mock_backend_configuration(TEST_BACKEND).await;

    let job_id = generate_uuid();
    let _job_submit_mock = da_server.mock_run_job(&job_id).await;
    
    // Mock job failure
    let _status_failed_mock = da_server.mock_job_status_failed(&job_id).await;
    let _job_details_mock = da_server.mock_job_details(&job_id, "Failed").await;

    let _s3_put_mock = s3_server.mock_put_object("test-key").await;

    let mut qrmi = IBMDirectAccess::new(TEST_BACKEND).expect("Failed to create QRMI instance");
    let _token = qrmi.acquire().await.expect("Failed to acquire");

    let payload = Payload::QiskitPrimitive {
        input: fixtures::estimator_v2_input(),
        program_id: "estimator".to_string(),
    };

    let task_id = qrmi.task_start(payload).await.expect("Failed to start task");

    // Check status - should be Failed
    let status = qrmi.task_status(&task_id).await.expect("Failed to get status");
    assert!(
        matches!(status, TaskStatus::Failed),
        "Job should have failed status"
    );

    cleanup_da_env_vars();
}

/// Test: Job cancellation
#[tokio::test]
async fn test_job_cancellation() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;

    setup_da_env_vars(&da_server.url(), &iam_server.url(), &s3_server.url());

    let _iam_mock = iam_server.mock_token_generation().await;
    let _backend_list_mock = da_server.mock_list_backends(TEST_BACKEND).await;
    let _backend_config_mock = da_server.mock_backend_configuration(TEST_BACKEND).await;

    let job_id = generate_uuid();
    let _job_submit_mock = da_server.mock_run_job(&job_id).await;
    let _status_running_mock = da_server.mock_job_status_running(&job_id).await;
    
    // Mock job cancellation
    let _cancel_mock = da_server.mock_cancel_job(&job_id).await;
    let _delete_mock = da_server.mock_delete_job(&job_id).await;

    let _s3_put_mock = s3_server.mock_put_object("test-key").await;

    let mut qrmi = IBMDirectAccess::new(TEST_BACKEND).expect("Failed to create QRMI instance");
    let _token = qrmi.acquire().await.expect("Failed to acquire");

    let payload = Payload::QiskitPrimitive {
        input: fixtures::estimator_v2_input(),
        program_id: "estimator".to_string(),
    };

    let task_id = qrmi.task_start(payload).await.expect("Failed to start task");

    // Cancel the job
    let result = qrmi.task_stop(&task_id).await;
    assert!(result.is_ok(), "Job cancellation should succeed");

    cleanup_da_env_vars();
}

/// Test: Target retrieval
#[tokio::test]
async fn test_target_retrieval() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;

    setup_da_env_vars(&da_server.url(), &iam_server.url(), &s3_server.url());

    let _iam_mock = iam_server.mock_token_generation().await;
    let _backend_config_mock = da_server.mock_backend_configuration(TEST_BACKEND).await;
    let _backend_props_mock = da_server.mock_backend_properties(TEST_BACKEND).await;

    let mut qrmi = IBMDirectAccess::new(TEST_BACKEND).expect("Failed to create QRMI instance");

    // Test target retrieval
    let target = qrmi.target().await.expect("Failed to get target");
    assert!(!target.value.is_empty(), "Target should not be empty");

    // Verify target contains expected backend information
    let target_json: serde_json::Value = serde_json::from_str(&target.value)
        .expect("Target should be valid JSON");
    assert!(target_json.get("configuration").is_some(), "Target should contain configuration");
    assert!(target_json.get("properties").is_some(), "Target should contain properties");

    cleanup_da_env_vars();
}

/// Test: Metadata retrieval
#[tokio::test]
async fn test_metadata_retrieval() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut da_server = mock_servers::MockDirectAccessServer::new().await;
    let mut s3_server = mock_servers::MockS3Server::new().await;

    setup_da_env_vars(&da_server.url(), &iam_server.url(), &s3_server.url());

    let _iam_mock = iam_server.mock_token_generation().await;

    let mut qrmi = IBMDirectAccess::new(TEST_BACKEND).expect("Failed to create QRMI instance");

    // Test metadata retrieval
    let metadata = qrmi.metadata().await.unwrap();
    assert!(!metadata.is_empty(), "Metadata should not be empty");

    // Verify metadata contains expected keys
    assert!(metadata.contains_key("backend_name"), "Metadata should contain backend_name");

    cleanup_da_env_vars();
}

// Made with Bob
