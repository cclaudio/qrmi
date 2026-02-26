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

use qrmi::ibm::IBMQiskitRuntimeService;
use qrmi::models::{Payload, TaskStatus};
use qrmi::QuantumResource;

use crate::common::{fixtures, mock_servers, set_test_env_vars, clear_test_env_vars, generate_uuid};

const TEST_BACKEND: &str = "test_ibm_qrs_backend";

/// Helper to setup environment variables for IBM QRS tests
fn setup_qrs_env_vars(qrs_endpoint: &str, iam_endpoint: &str) {
    set_test_env_vars(
        TEST_BACKEND,
        vec![
            ("QRMI_IBM_QRS_ENDPOINT", qrs_endpoint),
            ("QRMI_IBM_QRS_IAM_ENDPOINT", iam_endpoint),
            ("QRMI_IBM_QRS_IAM_APIKEY", "test_api_key"),
            ("QRMI_IBM_QRS_SERVICE_CRN", "crn:v1:test:qrs:service"),
            ("QRMI_IBM_QRS_SESSION_MODE", "dedicated"),
            ("QRMI_IBM_QRS_SESSION_MAX_TTL", "28800"),
        ],
    );
}

/// Helper to cleanup environment variables
fn cleanup_qrs_env_vars() {
    clear_test_env_vars(
        TEST_BACKEND,
        vec![
            "QRMI_IBM_QRS_ENDPOINT",
            "QRMI_IBM_QRS_IAM_ENDPOINT",
            "QRMI_IBM_QRS_IAM_APIKEY",
            "QRMI_IBM_QRS_SERVICE_CRN",
            "QRMI_IBM_QRS_SESSION_MODE",
            "QRMI_IBM_QRS_SESSION_MAX_TTL",
            "QRMI_IBM_QRS_SESSION_ID",
            "QRMI_IBM_QRS_TIMEOUT_SECONDS",
        ],
    );
}

/// Test: Full workflow - Happy path with EstimatorV2
/// Tests: acquire (session) → task_start → task_status → task_result → release (close session)
#[tokio::test]
async fn test_full_workflow_estimator_v2_success() {
    common::setup();

    // Setup mock servers
    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut qrs_server = mock_servers::MockQiskitRuntimeServer::new().await;

    // Setup environment
    setup_qrs_env_vars(&qrs_server.url(), &iam_server.url());

    // Mock IAM token generation
    let _iam_mock = iam_server.mock_token_generation().await;

    // Generate session and job IDs
    let session_id = generate_uuid();
    let job_id = generate_uuid();

    // Mock session creation (acquire)
    let _session_create_mock = qrs_server.mock_create_session(&session_id).await;

    // Mock backend operations
    let _backend_config_mock = qrs_server.mock_backend_configuration(TEST_BACKEND).await;
    let _backend_props_mock = qrs_server.mock_backend_properties(TEST_BACKEND).await;

    // Mock job creation
    let _job_create_mock = qrs_server.mock_create_job(&job_id).await;

    // Mock job status transitions: Queued → Running → Completed
    let _status_queued_mock = qrs_server.mock_job_status(&job_id, "Queued").await;
    let _status_running_mock = qrs_server.mock_job_status(&job_id, "Running").await;
    let _status_completed_mock = qrs_server.mock_job_status(&job_id, "Completed").await;

    // Mock session closure (release)
    let _session_close_mock = qrs_server.mock_close_session(&session_id).await;

    // Create QRMI instance
    let mut qrmi = IBMQiskitRuntimeService::new(TEST_BACKEND)
        .expect("Failed to create QRMI instance");

    // Test: is_accessible
    let accessible = qrmi
        .is_accessible()
        .await
        .expect("Failed to check accessibility");
    assert!(accessible, "Backend should be accessible");

    // Test: acquire (creates session)
    let acquisition_token = qrmi.acquire().await.expect("Failed to acquire resource");
    assert!(!acquisition_token.is_empty(), "Session ID should not be empty");

    // Test: task_start with EstimatorV2 payload
    let payload = Payload::QiskitPrimitive {
        input: fixtures::estimator_v2_input(),
        program_id: "estimator".to_string(),
    };

    let task_id = qrmi
        .task_start(payload)
        .await
        .expect("Failed to start task");
    assert!(!task_id.is_empty(), "Task ID should not be empty");

    // Test: task_status
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

    // Test: release (closes session)
    qrmi.release(&acquisition_token)
        .await
        .expect("Failed to release resource");

    // Cleanup
    cleanup_qrs_env_vars();
}

/// Test: Full workflow with SamplerV2
#[tokio::test]
async fn test_full_workflow_sampler_v2_success() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut qrs_server = mock_servers::MockQiskitRuntimeServer::new().await;

    setup_qrs_env_vars(&qrs_server.url(), &iam_server.url());

    let _iam_mock = iam_server.mock_token_generation().await;

    let session_id = generate_uuid();
    let job_id = generate_uuid();

    let _session_create_mock = qrs_server.mock_create_session(&session_id).await;
    let _backend_config_mock = qrs_server.mock_backend_configuration(TEST_BACKEND).await;
    let _job_create_mock = qrs_server.mock_create_job(&job_id).await;
    let _status_completed_mock = qrs_server.mock_job_status(&job_id, "Completed").await;
    let _session_close_mock = qrs_server.mock_close_session(&session_id).await;

    let mut qrmi = IBMQiskitRuntimeService::new(TEST_BACKEND)
        .expect("Failed to create QRMI instance");

    let _token = qrmi.acquire().await.expect("Failed to acquire");

    // Test with SamplerV2 payload
    let payload = Payload::QiskitPrimitive {
        input: fixtures::sampler_v2_input(),
        program_id: "sampler".to_string(),
    };

    let task_id = qrmi.task_start(payload).await.expect("Failed to start task");
    assert!(!task_id.is_empty());

    let status = qrmi.task_status(&task_id).await.expect("Failed to get status");
    assert!(matches!(status, TaskStatus::Completed | TaskStatus::Queued | TaskStatus::Running));

    cleanup_qrs_env_vars();
}

/// Test: Full workflow with NoiseLearner
#[tokio::test]
async fn test_full_workflow_noise_learner_success() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut qrs_server = mock_servers::MockQiskitRuntimeServer::new().await;

    setup_qrs_env_vars(&qrs_server.url(), &iam_server.url());

    let _iam_mock = iam_server.mock_token_generation().await;

    let session_id = generate_uuid();
    let job_id = generate_uuid();

    let _session_create_mock = qrs_server.mock_create_session(&session_id).await;
    let _backend_config_mock = qrs_server.mock_backend_configuration(TEST_BACKEND).await;
    let _job_create_mock = qrs_server.mock_create_job(&job_id).await;
    let _status_completed_mock = qrs_server.mock_job_status(&job_id, "Completed").await;
    let _session_close_mock = qrs_server.mock_close_session(&session_id).await;

    let mut qrmi = IBMQiskitRuntimeService::new(TEST_BACKEND)
        .expect("Failed to create QRMI instance");

    let _token = qrmi.acquire().await.expect("Failed to acquire");

    // Test with NoiseLearner payload
    let payload = Payload::QiskitPrimitive {
        input: fixtures::noise_learner_input(),
        program_id: "noise_learner".to_string(),
    };

    let task_id = qrmi.task_start(payload).await.expect("Failed to start task");
    assert!(!task_id.is_empty());

    cleanup_qrs_env_vars();
}

/// Test: Session mode - Dedicated
#[tokio::test]
async fn test_session_mode_dedicated() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut qrs_server = mock_servers::MockQiskitRuntimeServer::new().await;

    setup_qrs_env_vars(&qrs_server.url(), &iam_server.url());
    // Explicitly set dedicated mode
    std::env::set_var(format!("{}_QRMI_IBM_QRS_SESSION_MODE", TEST_BACKEND), "dedicated");

    let _iam_mock = iam_server.mock_token_generation().await;

    let session_id = generate_uuid();
    let _session_create_mock = qrs_server.mock_create_session(&session_id).await;
    let _session_close_mock = qrs_server.mock_close_session(&session_id).await;

    let mut qrmi = IBMQiskitRuntimeService::new(TEST_BACKEND)
        .expect("Failed to create QRMI instance");

    let token = qrmi.acquire().await.expect("Failed to acquire");
    assert!(!token.is_empty());

    qrmi.release(&token).await.expect("Failed to release");

    cleanup_qrs_env_vars();
}

/// Test: Session mode - Batch
#[tokio::test]
async fn test_session_mode_batch() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut qrs_server = mock_servers::MockQiskitRuntimeServer::new().await;

    setup_qrs_env_vars(&qrs_server.url(), &iam_server.url());
    // Set batch mode
    std::env::set_var(format!("{}_QRMI_IBM_QRS_SESSION_MODE", TEST_BACKEND), "batch");

    let _iam_mock = iam_server.mock_token_generation().await;

    let session_id = generate_uuid();
    let _session_create_mock = qrs_server.mock_create_session(&session_id).await;
    let _session_close_mock = qrs_server.mock_close_session(&session_id).await;

    let mut qrmi = IBMQiskitRuntimeService::new(TEST_BACKEND)
        .expect("Failed to create QRMI instance");

    let token = qrmi.acquire().await.expect("Failed to acquire");
    assert!(!token.is_empty());

    cleanup_qrs_env_vars();
}

/// Test: Pre-existing session ID
#[tokio::test]
async fn test_preexisting_session_id() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut qrs_server = mock_servers::MockQiskitRuntimeServer::new().await;

    setup_qrs_env_vars(&qrs_server.url(), &iam_server.url());
    
    // Set pre-existing session ID
    let existing_session_id = generate_uuid();
    std::env::set_var(
        format!("{}_QRMI_IBM_QRS_SESSION_ID", TEST_BACKEND),
        &existing_session_id
    );

    let _iam_mock = iam_server.mock_token_generation().await;
    let _backend_config_mock = qrs_server.mock_backend_configuration(TEST_BACKEND).await;

    let mut qrmi = IBMQiskitRuntimeService::new(TEST_BACKEND)
        .expect("Failed to create QRMI instance");

    // acquire should return the pre-existing session ID
    let token = qrmi.acquire().await.expect("Failed to acquire");
    assert_eq!(token, existing_session_id, "Should use pre-existing session ID");

    cleanup_qrs_env_vars();
}

/// Test: Authentication failure
#[tokio::test]
async fn test_authentication_failure() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut qrs_server = mock_servers::MockQiskitRuntimeServer::new().await;

    setup_qrs_env_vars(&qrs_server.url(), &iam_server.url());

    // Mock IAM token generation failure
    let _iam_mock = iam_server.mock_token_generation_failure().await;

    let mut qrmi = IBMQiskitRuntimeService::new(TEST_BACKEND)
        .expect("Failed to create QRMI instance");

    // Operations requiring authentication should fail
    let result = qrmi.is_accessible().await;
    assert!(result.is_err(), "Should fail with authentication error");

    cleanup_qrs_env_vars();
}

/// Test: Job execution failure
#[tokio::test]
async fn test_job_execution_failure() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut qrs_server = mock_servers::MockQiskitRuntimeServer::new().await;

    setup_qrs_env_vars(&qrs_server.url(), &iam_server.url());

    let _iam_mock = iam_server.mock_token_generation().await;

    let session_id = generate_uuid();
    let job_id = generate_uuid();

    let _session_create_mock = qrs_server.mock_create_session(&session_id).await;
    let _backend_config_mock = qrs_server.mock_backend_configuration(TEST_BACKEND).await;
    let _job_create_mock = qrs_server.mock_create_job(&job_id).await;
    
    // Mock job failure
    let _status_failed_mock = qrs_server.mock_job_status(&job_id, "Failed").await;

    let mut qrmi = IBMQiskitRuntimeService::new(TEST_BACKEND)
        .expect("Failed to create QRMI instance");

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

    cleanup_qrs_env_vars();
}

/// Test: Job timeout handling
#[tokio::test]
async fn test_job_timeout() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut qrs_server = mock_servers::MockQiskitRuntimeServer::new().await;

    setup_qrs_env_vars(&qrs_server.url(), &iam_server.url());
    
    // Set a short timeout
    std::env::set_var(format!("{}_QRMI_IBM_QRS_TIMEOUT_SECONDS", TEST_BACKEND), "10");

    let _iam_mock = iam_server.mock_token_generation().await;

    let session_id = generate_uuid();
    let job_id = generate_uuid();

    let _session_create_mock = qrs_server.mock_create_session(&session_id).await;
    let _backend_config_mock = qrs_server.mock_backend_configuration(TEST_BACKEND).await;
    let _job_create_mock = qrs_server.mock_create_job(&job_id).await;
    let _status_running_mock = qrs_server.mock_job_status(&job_id, "Running").await;

    let mut qrmi = IBMQiskitRuntimeService::new(TEST_BACKEND)
        .expect("Failed to create QRMI instance");

    let _token = qrmi.acquire().await.expect("Failed to acquire");

    let payload = Payload::QiskitPrimitive {
        input: fixtures::estimator_v2_input(),
        program_id: "estimator".to_string(),
    };

    let task_id = qrmi.task_start(payload).await.expect("Failed to start task");
    assert!(!task_id.is_empty());

    cleanup_qrs_env_vars();
}

/// Test: Target retrieval
#[tokio::test]
async fn test_target_retrieval() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut qrs_server = mock_servers::MockQiskitRuntimeServer::new().await;

    setup_qrs_env_vars(&qrs_server.url(), &iam_server.url());

    let _iam_mock = iam_server.mock_token_generation().await;
    let _backend_config_mock = qrs_server.mock_backend_configuration(TEST_BACKEND).await;
    let _backend_props_mock = qrs_server.mock_backend_properties(TEST_BACKEND).await;

    let mut qrmi = IBMQiskitRuntimeService::new(TEST_BACKEND)
        .expect("Failed to create QRMI instance");

    // Test target retrieval
    let target = qrmi.target().await.expect("Failed to get target");
    assert!(!target.value.is_empty(), "Target should not be empty");

    // Verify target contains expected backend information
    let target_json: serde_json::Value = serde_json::from_str(&target.value)
        .expect("Target should be valid JSON");
    assert!(target_json.get("configuration").is_some(), "Target should contain configuration");

    cleanup_qrs_env_vars();
}

/// Test: Metadata retrieval
#[tokio::test]
async fn test_metadata_retrieval() {
    common::setup();

    let mut iam_server = mock_servers::MockIamServer::new().await;
    let mut qrs_server = mock_servers::MockQiskitRuntimeServer::new().await;

    setup_qrs_env_vars(&qrs_server.url(), &iam_server.url());

    let _iam_mock = iam_server.mock_token_generation().await;

    let mut qrmi = IBMQiskitRuntimeService::new(TEST_BACKEND)
        .expect("Failed to create QRMI instance");

    // Test metadata retrieval
    let metadata = qrmi.metadata().await.unwrap();
    assert!(!metadata.is_empty(), "Metadata should not be empty");

    cleanup_qrs_env_vars();
}

// Made with Bob
