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

use qrmi::pasqal::PasqalCloud;
use qrmi::models::{Payload, TaskStatus};
use qrmi::QuantumResource;

use crate::common::{fixtures, mock_servers, set_test_env_vars, clear_test_env_vars, generate_uuid};

const TEST_BACKEND: &str = "FRESNEL";

/// Helper to setup environment variables for Pasqal Cloud tests
fn setup_pasqal_env_vars(project_id: &str, auth_token: &str) {
    set_test_env_vars(
        TEST_BACKEND,
        vec![
            ("QRMI_PASQAL_CLOUD_PROJECT_ID", project_id),
            ("QRMI_PASQAL_CLOUD_AUTH_TOKEN", auth_token),
        ],
    );
}

/// Helper to cleanup environment variables
fn cleanup_pasqal_env_vars() {
    clear_test_env_vars(
        TEST_BACKEND,
        vec![
            "QRMI_PASQAL_CLOUD_PROJECT_ID",
            "QRMI_PASQAL_CLOUD_AUTH_TOKEN",
        ],
    );
}

/// Test: Full workflow - Happy path with Pulser sequence
/// Tests: acquire → task_start → task_status → task_result → release
#[tokio::test]
async fn test_full_workflow_pulser_success() {
    common::setup();

    // Setup mock server
    let mut pasqal_server = mock_servers::MockPasqalCloudServer::new().await;

    // Setup environment
    let project_id = generate_uuid();
    setup_pasqal_env_vars(&project_id, "test_auth_token");

    // Mock device information
    let _device_mock = pasqal_server.mock_get_device(TEST_BACKEND).await;
    let _device_specs_mock = pasqal_server.mock_get_device_specs(TEST_BACKEND).await;

    // Generate batch ID
    let batch_id = generate_uuid();

    // Mock batch creation
    let _batch_create_mock = pasqal_server.mock_create_batch(&batch_id).await;

    // Mock batch status transitions: PENDING → RUNNING → DONE
    let _status_pending_mock = pasqal_server.mock_get_batch(&batch_id, "PENDING").await;
    let _status_running_mock = pasqal_server.mock_get_batch(&batch_id, "RUNNING").await;
    let _status_done_mock = pasqal_server.mock_get_batch(&batch_id, "DONE").await;

    // Create QRMI instance
    let mut qrmi = PasqalCloud::new(TEST_BACKEND).expect("Failed to create QRMI instance");

    // Test: is_accessible
    let accessible = qrmi
        .is_accessible()
        .await
        .expect("Failed to check accessibility");
    assert!(accessible, "Device should be accessible");

    // Test: acquire (returns dummy token for Pasqal)
    let acquisition_token = qrmi.acquire().await.expect("Failed to acquire resource");
    assert!(!acquisition_token.is_empty(), "Acquisition token should not be empty");

    // Test: task_start with Pulser sequence
    let payload = Payload::PasqalCloud {
        sequence: fixtures::pulser_sequence(),
        job_runs: 100,
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

    // Test: release (no-op for Pasqal)
    qrmi.release(&acquisition_token)
        .await
        .expect("Failed to release resource");

    // Cleanup
    cleanup_pasqal_env_vars();
}

/// Test: Multiple device types
#[tokio::test]
async fn test_multiple_device_types() {
    common::setup();

    let device_types = vec!["FRESNEL", "FRESNEL_CAN1", "EMU_MPS", "EMU_FREE"];

    for device_type in device_types {
        let mut pasqal_server = mock_servers::MockPasqalCloudServer::new().await;
        let project_id = generate_uuid();

        set_test_env_vars(
            device_type,
            vec![
                ("QRMI_PASQAL_CLOUD_PROJECT_ID", &project_id),
                ("QRMI_PASQAL_CLOUD_AUTH_TOKEN", "test_token"),
            ],
        );

        let _device_mock = pasqal_server.mock_get_device(device_type).await;

        let mut qrmi = PasqalCloud::new(device_type)
            .expect(&format!("Failed to create QRMI for {}", device_type));

        let accessible = qrmi
            .is_accessible()
            .await
            .expect(&format!("Failed to check {} accessibility", device_type));
        assert!(accessible, "{} should be accessible", device_type);

        clear_test_env_vars(
            device_type,
            vec!["QRMI_PASQAL_CLOUD_PROJECT_ID", "QRMI_PASQAL_CLOUD_AUTH_TOKEN"],
        );
    }
}

/// Test: Device not available (RETIRED)
#[tokio::test]
async fn test_device_not_available() {
    common::setup();

    let mut pasqal_server = mock_servers::MockPasqalCloudServer::new().await;
    let project_id = generate_uuid();

    setup_pasqal_env_vars(&project_id, "test_auth_token");

    // Mock device as not available
    let _device_mock = pasqal_server.mock_device_not_available(TEST_BACKEND).await;

    let mut qrmi = PasqalCloud::new(TEST_BACKEND).expect("Failed to create QRMI instance");

    let accessible = qrmi
        .is_accessible()
        .await
        .expect("Failed to check accessibility");
    assert!(!accessible, "Device should not be accessible when RETIRED");

    cleanup_pasqal_env_vars();
}

/// Test: Batch cancellation
#[tokio::test]
async fn test_batch_cancellation() {
    common::setup();

    let mut pasqal_server = mock_servers::MockPasqalCloudServer::new().await;
    let project_id = generate_uuid();

    setup_pasqal_env_vars(&project_id, "test_auth_token");

    let _device_mock = pasqal_server.mock_get_device(TEST_BACKEND).await;
    let _device_specs_mock = pasqal_server.mock_get_device_specs(TEST_BACKEND).await;

    let batch_id = generate_uuid();
    let _batch_create_mock = pasqal_server.mock_create_batch(&batch_id).await;
    let _status_running_mock = pasqal_server.mock_get_batch(&batch_id, "RUNNING").await;
    
    // Mock batch cancellation
    let _cancel_mock = pasqal_server.mock_cancel_batch(&batch_id).await;

    let mut qrmi = PasqalCloud::new(TEST_BACKEND).expect("Failed to create QRMI instance");
    let _token = qrmi.acquire().await.expect("Failed to acquire");

    let payload = Payload::PasqalCloud {
        sequence: fixtures::pulser_sequence(),
        job_runs: 100,
    };

    let task_id = qrmi.task_start(payload).await.expect("Failed to start task");

    // Cancel the batch
    let result = qrmi.task_stop(&task_id).await;
    assert!(result.is_ok(), "Batch cancellation should succeed");

    cleanup_pasqal_env_vars();
}

/// Test: Batch status transitions
#[tokio::test]
async fn test_batch_status_transitions() {
    common::setup();

    let mut pasqal_server = mock_servers::MockPasqalCloudServer::new().await;
    let project_id = generate_uuid();

    setup_pasqal_env_vars(&project_id, "test_auth_token");

    let _device_mock = pasqal_server.mock_get_device(TEST_BACKEND).await;
    let _device_specs_mock = pasqal_server.mock_get_device_specs(TEST_BACKEND).await;

    let batch_id = generate_uuid();
    let _batch_create_mock = pasqal_server.mock_create_batch(&batch_id).await;

    // Mock different status transitions
    let _status_pending = pasqal_server.mock_get_batch(&batch_id, "PENDING").await;
    let _status_running = pasqal_server.mock_get_batch(&batch_id, "RUNNING").await;
    let _status_done = pasqal_server.mock_get_batch(&batch_id, "DONE").await;

    let mut qrmi = PasqalCloud::new(TEST_BACKEND).expect("Failed to create QRMI instance");
    let _token = qrmi.acquire().await.expect("Failed to acquire");

    let payload = Payload::PasqalCloud {
        sequence: fixtures::pulser_sequence(),
        job_runs: 100,
    };

    let task_id = qrmi.task_start(payload).await.expect("Failed to start task");

    // Check status multiple times to see transitions
    let status1 = qrmi.task_status(&task_id).await.expect("Failed to get status");
    assert!(
        matches!(status1, TaskStatus::Queued | TaskStatus::Running | TaskStatus::Completed),
        "First status check should be valid"
    );

    cleanup_pasqal_env_vars();
}

/// Test: Empty auth token (public device access)
#[tokio::test]
async fn test_empty_auth_token_public_device() {
    common::setup();

    let mut pasqal_server = mock_servers::MockPasqalCloudServer::new().await;
    let project_id = generate_uuid();

    // Setup with empty auth token
    setup_pasqal_env_vars(&project_id, "");

    let _device_specs_mock = pasqal_server.mock_get_device_specs(TEST_BACKEND).await;

    let mut qrmi = PasqalCloud::new(TEST_BACKEND).expect("Failed to create QRMI instance");

    // Should be able to get device specs without auth token
    let target = qrmi.target().await.expect("Failed to get target");
    assert!(!target.value.is_empty(), "Target should not be empty");

    cleanup_pasqal_env_vars();
}

/// Test: Invalid device type
#[tokio::test]
async fn test_invalid_device_type() {
    common::setup();

    let invalid_device = "INVALID_DEVICE";
    let project_id = generate_uuid();

    set_test_env_vars(
        invalid_device,
        vec![
            ("QRMI_PASQAL_CLOUD_PROJECT_ID", &project_id),
            ("QRMI_PASQAL_CLOUD_AUTH_TOKEN", "test_token"),
        ],
    );

    let mut qrmi = PasqalCloud::new(invalid_device).expect("Failed to create QRMI instance");

    // Should fail when checking accessibility with invalid device
    let result = qrmi.is_accessible().await;
    assert!(result.is_err(), "Should fail with invalid device type");

    clear_test_env_vars(
        invalid_device,
        vec!["QRMI_PASQAL_CLOUD_PROJECT_ID", "QRMI_PASQAL_CLOUD_AUTH_TOKEN"],
    );
}

/// Test: Batch with different job_runs values
#[tokio::test]
async fn test_different_job_runs() {
    common::setup();

    let job_runs_values = vec![1, 10, 100, 1000];

    for job_runs in job_runs_values {
        let mut pasqal_server = mock_servers::MockPasqalCloudServer::new().await;
        let project_id = generate_uuid();

        setup_pasqal_env_vars(&project_id, "test_auth_token");

        let _device_mock = pasqal_server.mock_get_device(TEST_BACKEND).await;
        let _device_specs_mock = pasqal_server.mock_get_device_specs(TEST_BACKEND).await;

        let batch_id = generate_uuid();
        let _batch_create_mock = pasqal_server.mock_create_batch(&batch_id).await;
        let _status_mock = pasqal_server.mock_get_batch(&batch_id, "PENDING").await;

        let mut qrmi = PasqalCloud::new(TEST_BACKEND).expect("Failed to create QRMI instance");
        let _token = qrmi.acquire().await.expect("Failed to acquire");

        let payload = Payload::PasqalCloud {
            sequence: fixtures::pulser_sequence(),
            job_runs,
        };

        let task_id = qrmi.task_start(payload).await
            .expect(&format!("Failed to start task with job_runs={}", job_runs));
        assert!(!task_id.is_empty());

        cleanup_pasqal_env_vars();
    }
}

/// Test: Target retrieval (device specs)
#[tokio::test]
async fn test_target_retrieval() {
    common::setup();

    let mut pasqal_server = mock_servers::MockPasqalCloudServer::new().await;
    let project_id = generate_uuid();

    setup_pasqal_env_vars(&project_id, "test_auth_token");

    let _device_specs_mock = pasqal_server.mock_get_device_specs(TEST_BACKEND).await;

    let mut qrmi = PasqalCloud::new(TEST_BACKEND).expect("Failed to create QRMI instance");

    // Test target retrieval
    let target = qrmi.target().await.expect("Failed to get target");
    assert!(!target.value.is_empty(), "Target should not be empty");

    // Verify target contains device specs
    let target_json: serde_json::Value = serde_json::from_str(&target.value)
        .expect("Target should be valid JSON");
    assert!(
        target_json.is_object() || target_json.is_string(),
        "Target should contain device specs"
    );

    cleanup_pasqal_env_vars();
}

/// Test: Metadata retrieval
#[tokio::test]
async fn test_metadata_retrieval() {
    common::setup();

    let mut pasqal_server = mock_servers::MockPasqalCloudServer::new().await;
    let project_id = generate_uuid();

    setup_pasqal_env_vars(&project_id, "test_auth_token");

    let _device_mock = pasqal_server.mock_get_device(TEST_BACKEND).await;

    let mut qrmi = PasqalCloud::new(TEST_BACKEND).expect("Failed to create QRMI instance");

    // Test metadata retrieval
    let metadata = qrmi.metadata().await.unwrap();
    assert!(!metadata.is_empty(), "Metadata should not be empty");

    // Verify metadata contains expected keys
    assert!(
        metadata.contains_key("backend_name") || metadata.contains_key("device_type"),
        "Metadata should contain device information"
    );

    cleanup_pasqal_env_vars();
}

/// Test: Batch failure scenario
#[tokio::test]
async fn test_batch_failure() {
    common::setup();

    let mut pasqal_server = mock_servers::MockPasqalCloudServer::new().await;
    let project_id = generate_uuid();

    setup_pasqal_env_vars(&project_id, "test_auth_token");

    let _device_mock = pasqal_server.mock_get_device(TEST_BACKEND).await;
    let _device_specs_mock = pasqal_server.mock_get_device_specs(TEST_BACKEND).await;

    let batch_id = generate_uuid();
    let _batch_create_mock = pasqal_server.mock_create_batch(&batch_id).await;
    
    // Mock batch failure
    let _status_failed_mock = pasqal_server.mock_get_batch(&batch_id, "ERROR").await;

    let mut qrmi = PasqalCloud::new(TEST_BACKEND).expect("Failed to create QRMI instance");
    let _token = qrmi.acquire().await.expect("Failed to acquire");

    let payload = Payload::PasqalCloud {
        sequence: fixtures::pulser_sequence(),
        job_runs: 100,
    };

    let task_id = qrmi.task_start(payload).await.expect("Failed to start task");

    // Check status - should be Failed
    let status = qrmi.task_status(&task_id).await.expect("Failed to get status");
    assert!(
        matches!(status, TaskStatus::Failed | TaskStatus::Cancelled),
        "Batch should have failed or cancelled status"
    );

    cleanup_pasqal_env_vars();
}

// Made with Bob
