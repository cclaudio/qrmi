# QRMI Python Integration Tests

This directory contains integration tests for the QRMI Python bindings.

## Overview

These tests verify that the Python bindings work correctly with the Rust implementation, including:
- Type conversion between Python and Rust
- Error handling and exception propagation  
- Memory management across the language boundary
- API completeness and correctness

## Test Structure

```
python/tests/
├── __init__.py                 # Package marker
├── conftest.py                 # Pytest fixtures and mock servers
├── test_python_bindings.py     # Main integration tests
├── pytest.ini                  # Pytest configuration
├── requirements-test.txt       # Test dependencies
└── README.md                   # This file
```

## Running Tests

### Prerequisites

1. **Build the Python module**:
   ```bash
   # From project root
   maturin develop --features pyo3
   ```

2. **Install test dependencies**:
   ```bash
   pip install -r python/tests/requirements-test.txt
   ```

### Run All Tests

```bash
# From project root
pytest python/tests/

# Or from python/tests directory
pytest
```

### Run Specific Tests

```bash
# Run specific test class
pytest python/tests/test_python_bindings.py::TestPythonBindingsBasic

# Run specific test
pytest python/tests/test_python_bindings.py::TestIBMDirectAccessPython::test_full_workflow_estimator

# Run with verbose output
pytest python/tests/ -v

# Run with coverage
pytest python/tests/ --cov=qrmi --cov-report=html
```

## Test Coverage

### Basic Binding Tests
- ✅ Module import and attribute checks
- ✅ ResourceType enum validation
- ✅ TaskStatus enum validation

### IBM Direct Access Tests (7 tests)
- ✅ Resource creation
- ✅ Accessibility checks
- ✅ Acquire and release
- ✅ Full workflow with Estimator
- ✅ Target retrieval
- ✅ Metadata retrieval
- ❌ Error handling

### IBM Qiskit Runtime Service Tests (2 tests)
- ✅ Resource creation
- ✅ Full workflow with Sampler

### Pasqal Cloud Tests (3 tests)
- ✅ Resource creation
- ✅ Full workflow with Pulser
- ✅ Target retrieval

### Edge Case Tests (4 tests)
- ❌ Invalid backend names
- ✅ Different payload types
- ✅ TaskResult structure
- ✅ Multiple resources

**Total: 19 Python integration tests**

## Mock Server Architecture

Tests use lightweight HTTP mock servers implemented in Python:
- **MockServer**: Generic HTTP server for mocking API responses
- **Fixtures**: Pytest fixtures for each provider (IAM, DA, QRS, Pasqal)
- **Automatic cleanup**: Servers are automatically shut down after tests

## Test Fixtures

### Server Fixtures
- `mock_iam_server` - Mock IBM IAM authentication server
- `mock_da_server` - Mock Direct Access API server
- `mock_qrs_server` - Mock Qiskit Runtime Service server
- `mock_pasqal_server` - Mock Pasqal Cloud API server

### Environment Fixtures
- `setup_da_env` - Configure environment for Direct Access tests
- `setup_qrs_env` - Configure environment for QRS tests
- `setup_pasqal_env` - Configure environment for Pasqal tests

### Data Fixtures
- `estimator_input` - Sample Estimator V2 input
- `sampler_input` - Sample Sampler V2 input
- `pulser_sequence` - Sample Pulser sequence

## Writing New Tests

### Example Test

```python
def test_my_feature(setup_da_env, estimator_input):
    """Test description"""
    backend, da_server, iam_server = setup_da_env
    
    # Setup mock responses
    da_server.add_response('GET', '/v1/backends', body={
        'backends': [{'name': backend, 'status': 'online'}]
    })
    
    # Create resource
    qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMDirectAccess)
    
    # Test operations
    result = qrmi_resource.some_operation()
    assert result is not None
```

### Adding Mock Responses

```python
# Add GET response
server.add_response('GET', '/path', status=200, body={'key': 'value'})

# Add POST response
server.add_response('POST', '/path', status=201, body={'id': '123'})

# Add error response
server.add_response('GET', '/path', status=404, body={'error': 'Not found'})
```

## Debugging Tests

### Enable Verbose Output

```bash
pytest python/tests/ -v -s
```

### Run Single Test with Debug

```bash
pytest python/tests/test_python_bindings.py::test_name -v -s --pdb
```

### Check Test Coverage

```bash
pytest python/tests/ --cov=qrmi --cov-report=term-missing
```

## CI/CD Integration

These tests are integrated into the GitHub Actions workflow:

```yaml
- name: Build Python module
  run: maturin develop --features pyo3

- name: Install test dependencies
  run: pip install -r python/tests/requirements-test.txt

- name: Run Python tests
  run: pytest python/tests/ -v
```

## Troubleshooting

### Module Not Found

If you get `ModuleNotFoundError: No module named 'qrmi'`:
```bash
# Build and install the module
maturin develop --features pyo3
```

### Import Errors

If you get import errors for test dependencies:
```bash
pip install -r python/tests/requirements-test.txt
```

### Mock Server Issues

If tests hang or fail with connection errors:
- Check that mock servers are properly shut down
- Verify port availability
- Check firewall settings

### Type Errors

Type checking errors from basedpyright/mypy are expected before building the module. They will resolve after running `maturin develop`.

## Best Practices

1. **Isolation**: Each test should be independent
2. **Cleanup**: Use fixtures for automatic cleanup
3. **Mocking**: Mock all external HTTP calls
4. **Assertions**: Use descriptive assertion messages
5. **Documentation**: Document test purpose and setup

## Contributing

When adding new tests:
1. Follow existing test patterns
2. Add appropriate fixtures
3. Update this README
4. Ensure tests pass locally
5. Add to CI/CD if needed

## References

- [Pytest Documentation](https://docs.pytest.org/)
- [QRMI Documentation](../../README.md)
- [Maturin Documentation](https://www.maturin.rs/)
- [PyO3 Documentation](https://pyo3.rs/)