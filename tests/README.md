# QRMI Integration Tests

This directory contains comprehensive integration tests for the Quantum Resource Management Interface (QRMI).

## Overview

The integration test suite validates end-to-end workflows for all supported quantum providers:

- **IBM Direct Access** - Direct access to IBM Quantum hardware
- **IBM Qiskit Runtime Service** - IBM's cloud-based quantum computing service
- **Pasqal Cloud** - Pasqal's neutral atom quantum computing platform

## Test Structure

```
tests/
├── lib.rs                          # Main test entry point
├── common/
│   ├── mod.rs                      # Common utilities and setup
│   ├── fixtures.rs                 # Test data and payloads
│   └── mock_servers.rs             # Mock HTTP servers for each provider
└── integration/
    ├── mod.rs                      # Integration test module
    ├── ibm_direct_access.rs        # IBM Direct Access tests
    ├── ibm_qrs.rs                  # IBM QRS tests
    └── pasqal_cloud.rs             # Pasqal Cloud tests
```

## Test Coverage

### Full Workflow Tests
Each provider includes tests for the complete QRMI workflow:
1. **acquire()** - Acquire quantum resource (session/lock)
2. **task_start()** - Submit quantum job
3. **task_status()** - Monitor job status
4. **task_result()** - Retrieve results
5. **task_stop()** - Cancel job (if needed)
6. **release()** - Release resource

### Scenario Coverage

#### IBM Direct Access
- ✅ Full workflow with Estimator V2
- ✅ Full workflow with Sampler V2
- ✅ Full workflow with Noise Learner
- ❌ Authentication failures
- ❌ Backend not accessible
- ❌ Job execution failures
- 🔄 Job cancellation
- 📊 Target and metadata retrieval

#### IBM Qiskit Runtime Service
- ✅ Full workflow with Estimator V2
- ✅ Full workflow with Sampler V2
- ✅ Full workflow with Noise Learner
- 🔐 Session modes (dedicated/batch)
- 🔐 Pre-existing session handling
- ❌ Authentication failures
- ❌ Job execution failures
- ⏱️ Job timeout handling
- 📊 Target and metadata retrieval

#### Pasqal Cloud
- ✅ Full workflow with Pulser sequences
- 🎯 Multiple device types (FRESNEL, EMU_MPS, etc.)
- ❌ Device not available (RETIRED)
- 🔄 Batch cancellation
- 📊 Batch status transitions
- 🔓 Public device access (empty auth token)
- ❌ Invalid device types
- 📊 Target and metadata retrieval

#### C FFI Bindings
- ✅ String memory management
- ✅ Null pointer handling
- ✅ Resource lifecycle (creation/destruction)
- ✅ Resource accessibility checks
- ✅ Full workflow through C API
- ✅ Target retrieval via C API
- ✅ Metadata retrieval via C API
- ❌ Error handling with invalid resources
- 📊 Last error retrieval
- 🔄 Payload memory management
- 🎯 Multiple resources simultaneously

## Running Tests

### Prerequisites

Ensure you have the required dependencies:
```bash
cargo build --tests
```

### Run All Integration Tests

```bash
cargo test --test lib
```

### Run Tests for Specific Provider

```bash
# IBM Direct Access tests
cargo test --test lib ibm_direct_access

# IBM Qiskit Runtime Service tests
cargo test --test lib ibm_qrs

# Pasqal Cloud tests
cargo test --test lib pasqal_cloud

# C FFI tests
cargo test --test lib c_ffi
```

### Run Specific Test

```bash
cargo test --test lib test_full_workflow_estimator_success
```

### Run with Output

```bash
cargo test --test lib -- --nocapture
```

### Run with Logging

```bash
RUST_LOG=debug cargo test --test lib -- --nocapture
```

## Test Architecture

### Mock Servers

Tests use `mockito` to create isolated HTTP mock servers for each provider:

- **MockIamServer** - IBM IAM authentication
- **MockDirectAccessServer** - IBM Direct Access API
- **MockQiskitRuntimeServer** - IBM QRS API
- **MockPasqalCloudServer** - Pasqal Cloud API
- **MockS3Server** - S3 storage operations

### Test Fixtures

Pre-defined test data in `common/fixtures.rs`:
- Sample quantum circuits and observables
- Backend configurations and properties
- Job results and logs
- Authentication tokens
- Device specifications

### Environment Variables

Tests set temporary environment variables for each backend:

**IBM Direct Access:**
```
{backend}_QRMI_IBM_DA_ENDPOINT
{backend}_QRMI_IBM_DA_IAM_ENDPOINT
{backend}_QRMI_IBM_DA_IAM_APIKEY
{backend}_QRMI_IBM_DA_SERVICE_CRN
{backend}_QRMI_IBM_DA_S3_ENDPOINT
{backend}_QRMI_IBM_DA_S3_BUCKET
{backend}_QRMI_IBM_DA_S3_REGION
```

**IBM QRS:**
```
{backend}_QRMI_IBM_QRS_ENDPOINT
{backend}_QRMI_IBM_QRS_IAM_ENDPOINT
{backend}_QRMI_IBM_QRS_IAM_APIKEY
{backend}_QRMI_IBM_QRS_SERVICE_CRN
{backend}_QRMI_IBM_QRS_SESSION_MODE
{backend}_QRMI_IBM_QRS_SESSION_MAX_TTL
```

**Pasqal Cloud:**
```
{backend}_QRMI_PASQAL_CLOUD_PROJECT_ID
{backend}_QRMI_PASQAL_CLOUD_AUTH_TOKEN
```

## Adding New Tests

### 1. Create Test Function

```rust
#[tokio::test]
async fn test_my_new_scenario() {
    common::setup();
    
    // Setup mock servers
    let mut server = mock_servers::MockDirectAccessServer::new().await;
    
    // Setup environment
    setup_da_env_vars(&server.url(), ...);
    
    // Create mocks
    let _mock = server.mock_operation().await;
    
    // Create QRMI instance
    let mut qrmi = IBMDirectAccess::new("test_backend")
        .expect("Failed to create QRMI");
    
    // Test operations
    let result = qrmi.some_operation().await;
    assert!(result.is_ok());
    
    // Cleanup
    cleanup_da_env_vars();
}
```

### 2. Add Mock Server Methods

Add new mock methods to `common/mock_servers.rs`:

```rust
pub async fn mock_new_operation(&mut self) -> Mock {
    self.server
        .mock("GET", "/new/endpoint")
        .with_status(200)
        .with_body(json!({...}).to_string())
        .create_async()
        .await
}
```

### 3. Add Test Fixtures

Add new test data to `common/fixtures.rs`:

```rust
pub fn new_test_data() -> serde_json::Value {
    json!({
        "field": "value"
    })
}
```

## Debugging Tests

### Enable Logging

```bash
RUST_LOG=trace cargo test --test lib test_name -- --nocapture
```

### Run Single Test

```bash
cargo test --test lib test_full_workflow_estimator_success -- --exact --nocapture
```

### Check Mock Server Interactions

The `mockito` library provides detailed output about HTTP interactions when tests fail.

## Best Practices

1. **Isolation** - Each test is independent with its own mock servers
2. **Cleanup** - Always cleanup environment variables after tests
3. **Assertions** - Use descriptive assertion messages
4. **Async** - Use `#[tokio::test]` for async tests
5. **Setup** - Call `common::setup()` at the start of each test
6. **Mocking** - Mock all external HTTP calls
7. **Fixtures** - Use shared fixtures for common test data

## Continuous Integration

### GitHub Actions Example

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run integration tests
        run: cargo test --test lib
```

## Troubleshooting

### Test Hangs
- Check for missing `.await` on async operations
- Verify mock server is properly configured
- Ensure environment variables are set correctly

### Authentication Errors
- Verify IAM mock is configured before operations
- Check token generation mock is created
- Ensure correct endpoint URLs

### Mock Not Matching
- Verify HTTP method (GET, POST, etc.)
- Check endpoint path matches exactly
- Ensure request body format is correct

## Contributing

When adding new tests:
1. Follow existing test patterns
2. Add comprehensive documentation
3. Include both success and failure scenarios
4. Update this README with new test coverage
5. Ensure tests are isolated and repeatable

## References

- [QRMI Documentation](../README.md)
- [Mockito Documentation](https://docs.rs/mockito/)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)