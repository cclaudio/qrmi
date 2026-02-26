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

use mockito::{Mock, Server, ServerGuard};
use serde_json::json;
use uuid::Uuid;

use super::fixtures;
use super::generate_random_string;

/// Mock server for IBM IAM authentication
pub struct MockIamServer {
    pub server: ServerGuard,
}

impl MockIamServer {
    pub async fn new() -> Self {
        let server = mockito::Server::new_async().await;
        Self { server }
    }

    /// Setup mock for token generation
    pub async fn mock_token_generation(&mut self) -> Mock {
        self.server
            .mock("POST", "/identity/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(fixtures::iam_token_response().to_string())
            .create_async()
            .await
    }

    /// Setup mock for token generation failure
    pub async fn mock_token_generation_failure(&mut self) -> Mock {
        self.server
            .mock("POST", "/identity/token")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "errorCode": "BXNIM0415E",
                    "errorMessage": "Provided API key could not be found.",
                    "context": {
                        "requestId": generate_random_string(20)
                    }
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    pub fn url(&self) -> String {
        self.server.url()
    }
}

/// Mock server for IBM Direct Access API
pub struct MockDirectAccessServer {
    pub server: ServerGuard,
}

impl MockDirectAccessServer {
    pub async fn new() -> Self {
        let server = mockito::Server::new_async().await;
        Self { server }
    }

    /// Setup mock for backend list
    pub async fn mock_list_backends(&mut self, backend_name: &str) -> Mock {
        self.server
            .mock("GET", "/v1/backends")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "backends": [
                        {
                            "name": backend_name,
                            "status": "online",
                            "version": "1.0.0"
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Setup mock for backend configuration
    pub async fn mock_backend_configuration(&mut self, backend_name: &str) -> Mock {
        self.server
            .mock("GET", format!("/v1/backends/{}/configuration", backend_name).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(fixtures::backend_configuration().to_string())
            .create_async()
            .await
    }

    /// Setup mock for backend properties
    pub async fn mock_backend_properties(&mut self, backend_name: &str) -> Mock {
        self.server
            .mock("GET", format!("/v1/backends/{}/properties", backend_name).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(fixtures::backend_properties().to_string())
            .create_async()
            .await
    }

    /// Setup mock for job submission
    pub async fn mock_run_job(&mut self, job_id: &str) -> Mock {
        self.server
            .mock("POST", "/v1/jobs")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": job_id,
                    "status": "Queued"
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Setup mock for job status - queued
    pub async fn mock_job_status_queued(&mut self, job_id: &str) -> Mock {
        self.server
            .mock("GET", format!("/v1/jobs/{}/status", job_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "status": "Queued"
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Setup mock for job status - running
    pub async fn mock_job_status_running(&mut self, job_id: &str) -> Mock {
        self.server
            .mock("GET", format!("/v1/jobs/{}/status", job_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "status": "Running"
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Setup mock for job status - completed
    pub async fn mock_job_status_completed(&mut self, job_id: &str) -> Mock {
        self.server
            .mock("GET", format!("/v1/jobs/{}/status", job_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "status": "Completed"
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Setup mock for job status - failed
    pub async fn mock_job_status_failed(&mut self, job_id: &str) -> Mock {
        self.server
            .mock("GET", format!("/v1/jobs/{}/status", job_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "status": "Failed",
                    "reason_code": 1517,
                    "reason_message": "Job execution failed"
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Setup mock for job details
    pub async fn mock_job_details(&mut self, job_id: &str, status: &str) -> Mock {
        self.server
            .mock("GET", format!("/v1/jobs/{}", job_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": job_id,
                    "program_id": "estimator",
                    "backend": "test_backend",
                    "status": status,
                    "created_time": "2025-02-25T12:00:00Z"
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Setup mock for job cancellation
    pub async fn mock_cancel_job(&mut self, job_id: &str) -> Mock {
        self.server
            .mock("POST", format!("/v1/jobs/{}/cancel", job_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({}).to_string())
            .create_async()
            .await
    }

    /// Setup mock for job deletion
    pub async fn mock_delete_job(&mut self, job_id: &str) -> Mock {
        self.server
            .mock("DELETE", format!("/v1/jobs/{}", job_id).as_str())
            .with_status(204)
            .create_async()
            .await
    }

    /// Setup mock for backend not found
    pub async fn mock_backend_not_found(&mut self, backend_name: &str) -> Mock {
        self.server
            .mock("GET", format!("/v1/backends/{}/configuration", backend_name).as_str())
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "status_code": 404,
                    "title": format!("Backend {} not found.", backend_name),
                    "trace": Uuid::new_v4().to_string(),
                    "errors": [
                        {
                            "code": "1216",
                            "message": format!("Backend {} not found.", backend_name)
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    pub fn url(&self) -> String {
        self.server.url()
    }
}

/// Mock server for IBM Qiskit Runtime Service
pub struct MockQiskitRuntimeServer {
    pub server: ServerGuard,
}

impl MockQiskitRuntimeServer {
    pub async fn new() -> Self {
        let server = mockito::Server::new_async().await;
        Self { server }
    }

    /// Setup mock for session creation
    pub async fn mock_create_session(&mut self, session_id: &str) -> Mock {
        self.server
            .mock("POST", "/sessions")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(fixtures::qrs_session_response(session_id).to_string())
            .create_async()
            .await
    }

    /// Setup mock for session closure
    pub async fn mock_close_session(&mut self, session_id: &str) -> Mock {
        self.server
            .mock("PATCH", format!("/sessions/{}", session_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": session_id,
                    "state": "closed"
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Setup mock for job creation
    pub async fn mock_create_job(&mut self, job_id: &str) -> Mock {
        self.server
            .mock("POST", "/jobs")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": job_id
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Setup mock for job status
    pub async fn mock_job_status(&mut self, job_id: &str, state: &str) -> Mock {
        self.server
            .mock("GET", format!("/jobs/{}", job_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": job_id,
                    "state": state,
                    "backend": "test_backend",
                    "program": {
                        "id": "estimator"
                    }
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Setup mock for backend configuration
    pub async fn mock_backend_configuration(&mut self, backend_name: &str) -> Mock {
        self.server
            .mock("GET", format!("/backends/{}/configuration", backend_name).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(fixtures::backend_configuration().to_string())
            .create_async()
            .await
    }

    /// Setup mock for backend properties
    pub async fn mock_backend_properties(&mut self, backend_name: &str) -> Mock {
        self.server
            .mock("GET", format!("/backends/{}/properties", backend_name).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(fixtures::backend_properties().to_string())
            .create_async()
            .await
    }

    pub fn url(&self) -> String {
        self.server.url()
    }
}

/// Mock server for Pasqal Cloud API
pub struct MockPasqalCloudServer {
    pub server: ServerGuard,
}

impl MockPasqalCloudServer {
    pub async fn new() -> Self {
        let server = mockito::Server::new_async().await;
        Self { server }
    }

    /// Setup mock for device information
    pub async fn mock_get_device(&mut self, device_type: &str) -> Mock {
        self.server
            .mock("GET", format!("/devices/{}", device_type).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(fixtures::pasqal_device_response().to_string())
            .create_async()
            .await
    }

    /// Setup mock for device specs
    pub async fn mock_get_device_specs(&mut self, device_type: &str) -> Mock {
        self.server
            .mock(
                "GET",
                format!("/core-fast/api/v1/devices/specs/{}", device_type).as_str(),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "specs": fixtures::pasqal_device_response()["specs"].to_string()
                    }
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Setup mock for batch creation
    pub async fn mock_create_batch(&mut self, batch_id: &str) -> Mock {
        self.server
            .mock("POST", "/batches")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(fixtures::pasqal_batch_response(batch_id, "PENDING").to_string())
            .create_async()
            .await
    }

    /// Setup mock for batch status
    pub async fn mock_get_batch(&mut self, batch_id: &str, status: &str) -> Mock {
        self.server
            .mock("GET", format!("/batches/{}", batch_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(fixtures::pasqal_batch_response(batch_id, status).to_string())
            .create_async()
            .await
    }

    /// Setup mock for batch cancellation
    pub async fn mock_cancel_batch(&mut self, batch_id: &str) -> Mock {
        self.server
            .mock("POST", format!("/batches/{}/cancel", batch_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(fixtures::pasqal_batch_response(batch_id, "CANCELED").to_string())
            .create_async()
            .await
    }

    /// Setup mock for device not available
    pub async fn mock_device_not_available(&mut self, device_type: &str) -> Mock {
        self.server
            .mock("GET", format!("/devices/{}", device_type).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": device_type,
                    "name": device_type,
                    "availability": "RETIRED",
                    "status": "UNAVAILABLE"
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    pub fn url(&self) -> String {
        self.server.url()
    }
}

/// Mock S3 server for storage operations
pub struct MockS3Server {
    pub server: ServerGuard,
}

impl MockS3Server {
    pub async fn new() -> Self {
        let server = mockito::Server::new_async().await;
        Self { server }
    }

    /// Setup mock for PUT operation (upload)
    pub async fn mock_put_object(&mut self, key: &str) -> Mock {
        self.server
            .mock("PUT", format!("/{}", key).as_str())
            .with_status(200)
            .create_async()
            .await
    }

    /// Setup mock for GET operation (download)
    pub async fn mock_get_object(&mut self, key: &str, content: &str) -> Mock {
        self.server
            .mock("GET", format!("/{}", key).as_str())
            .with_status(200)
            .with_body(content)
            .create_async()
            .await
    }

    pub fn url(&self) -> String {
        self.server.url()
    }
}

// Made with Bob
