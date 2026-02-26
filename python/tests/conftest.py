# This code is part of Qiskit.
#
# (C) Copyright IBM 2025
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Pytest configuration and fixtures for QRMI Python integration tests"""

import json
import os
import pytest
from http.server import HTTPServer, BaseHTTPRequestHandler
from threading import Thread
import time


class MockServerHandler(BaseHTTPRequestHandler):
    """HTTP request handler for mock servers"""
    
    # Class-level storage for mock responses
    responses = {}
    
    def log_message(self, format, *args):
        """Suppress default logging"""
        pass
    
    def do_GET(self):
        """Handle GET requests"""
        key = f"GET:{self.path}"
        if key in self.responses:
            response = self.responses[key]
            self.send_response(response.get('status', 200))
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps(response.get('body', {})).encode())
        else:
            self.send_response(404)
            self.end_headers()
    
    def do_POST(self):
        """Handle POST requests"""
        key = f"POST:{self.path}"
        if key in self.responses:
            response = self.responses[key]
            self.send_response(response.get('status', 200))
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps(response.get('body', {})).encode())
        else:
            self.send_response(404)
            self.end_headers()
    
    def do_PATCH(self):
        """Handle PATCH requests"""
        key = f"PATCH:{self.path}"
        if key in self.responses:
            response = self.responses[key]
            self.send_response(response.get('status', 200))
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps(response.get('body', {})).encode())
        else:
            self.send_response(404)
            self.end_headers()
    
    def do_DELETE(self):
        """Handle DELETE requests"""
        self.send_response(204)
        self.end_headers()


class MockServer:
    """Mock HTTP server for testing"""
    
    def __init__(self, port=0):
        self.server = HTTPServer(('localhost', port), MockServerHandler)
        self.port = self.server.server_address[1]
        self.thread = Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        time.sleep(0.1)  # Give server time to start
    
    def add_response(self, method, path, status=200, body=None):
        """Add a mock response"""
        key = f"{method}:{path}"
        MockServerHandler.responses[key] = {
            'status': status,
            'body': body or {}
        }
    
    def url(self):
        """Get server URL"""
        return f"http://localhost:{self.port}"
    
    def shutdown(self):
        """Shutdown the server"""
        self.server.shutdown()
        MockServerHandler.responses.clear()


@pytest.fixture
def mock_iam_server():
    """Fixture for mock IAM server"""
    server = MockServer()
    
    # Add IAM token response
    server.add_response('POST', '/identity/token', body={
        'access_token': 'test_access_token_12345',
        'refresh_token': 'test_refresh_token_67890',
        'token_type': 'Bearer',
        'expires_in': 3600,
        'expiration': 1740484800,
        'scope': 'ibm openid'
    })
    
    yield server
    server.shutdown()


@pytest.fixture
def mock_da_server():
    """Fixture for mock Direct Access server"""
    server = MockServer()
    yield server
    server.shutdown()


@pytest.fixture
def mock_qrs_server():
    """Fixture for mock Qiskit Runtime Service server"""
    server = MockServer()
    yield server
    server.shutdown()


@pytest.fixture
def mock_pasqal_server():
    """Fixture for mock Pasqal Cloud server"""
    server = MockServer()
    yield server
    server.shutdown()


@pytest.fixture
def setup_da_env(mock_da_server, mock_iam_server):
    """Setup environment variables for Direct Access tests"""
    backend = "test_py_da_backend"
    env_vars = {
        f"{backend}_QRMI_IBM_DA_ENDPOINT": mock_da_server.url(),
        f"{backend}_QRMI_IBM_DA_IAM_ENDPOINT": mock_iam_server.url(),
        f"{backend}_QRMI_IBM_DA_IAM_APIKEY": "test_api_key",
        f"{backend}_QRMI_IBM_DA_SERVICE_CRN": "crn:v1:test:service",
        f"{backend}_QRMI_IBM_DA_AWS_ACCESS_KEY_ID": "test_access_key",
        f"{backend}_QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY": "test_secret_key",
        f"{backend}_QRMI_IBM_DA_S3_ENDPOINT": "http://localhost:9000",
        f"{backend}_QRMI_IBM_DA_S3_BUCKET": "test-bucket",
        f"{backend}_QRMI_IBM_DA_S3_REGION": "us-east-1",
    }
    
    # Set environment variables
    for key, value in env_vars.items():
        os.environ[key] = value
    
    yield backend, mock_da_server, mock_iam_server
    
    # Cleanup
    for key in env_vars.keys():
        os.environ.pop(key, None)


@pytest.fixture
def setup_qrs_env(mock_qrs_server, mock_iam_server):
    """Setup environment variables for QRS tests"""
    backend = "test_py_qrs_backend"
    env_vars = {
        f"{backend}_QRMI_IBM_QRS_ENDPOINT": mock_qrs_server.url(),
        f"{backend}_QRMI_IBM_QRS_IAM_ENDPOINT": mock_iam_server.url(),
        f"{backend}_QRMI_IBM_QRS_IAM_APIKEY": "test_api_key",
        f"{backend}_QRMI_IBM_QRS_SERVICE_CRN": "crn:v1:test:qrs:service",
        f"{backend}_QRMI_IBM_QRS_SESSION_MODE": "dedicated",
        f"{backend}_QRMI_IBM_QRS_SESSION_MAX_TTL": "28800",
    }
    
    # Set environment variables
    for key, value in env_vars.items():
        os.environ[key] = value
    
    yield backend, mock_qrs_server, mock_iam_server
    
    # Cleanup
    for key in env_vars.keys():
        os.environ.pop(key, None)


@pytest.fixture
def setup_pasqal_env(mock_pasqal_server):
    """Setup environment variables for Pasqal tests"""
    backend = "FRESNEL"
    env_vars = {
        f"{backend}_QRMI_PASQAL_CLOUD_PROJECT_ID": "test-project-id",
        f"{backend}_QRMI_PASQAL_CLOUD_AUTH_TOKEN": "test_auth_token",
    }
    
    # Set environment variables
    for key, value in env_vars.items():
        os.environ[key] = value
    
    yield backend, mock_pasqal_server
    
    # Cleanup
    for key in env_vars.keys():
        os.environ.pop(key, None)


@pytest.fixture
def estimator_input():
    """Sample Estimator V2 input"""
    return json.dumps({
        "pubs": [{
            "circuit": {
                "num_qubits": 2,
                "data": [
                    ["h", [0], []],
                    ["cx", [0, 1], []]
                ]
            },
            "observables": [{
                "paulis": ["ZZ"],
                "coeffs": [1.0]
            }],
            "precision": 0.01
        }],
        "version": 2
    })


@pytest.fixture
def sampler_input():
    """Sample Sampler V2 input"""
    return json.dumps({
        "pubs": [{
            "circuit": {
                "num_qubits": 2,
                "data": [
                    ["h", [0], []],
                    ["cx", [0, 1], []],
                    ["measure", [0, 1], [0, 1]]
                ]
            },
            "shots": 1024
        }],
        "version": 2
    })


@pytest.fixture
def pulser_sequence():
    """Sample Pulser sequence"""
    return json.dumps({
        "sequence": {
            "device": "FRESNEL",
            "register": {
                "layout": {
                    "coordinates": [[0, 0], [5, 0]]
                }
            },
            "channels": {
                "rydberg_global": {
                    "initial_target": "all",
                    "pulses": [{
                        "amplitude": {
                            "kind": "constant",
                            "duration": 1000,
                            "value": 1.0
                        },
                        "detuning": {
                            "kind": "constant",
                            "duration": 1000,
                            "value": 0.0
                        },
                        "phase": 0.0
                    }]
                }
            }
        }
    })

# Made with Bob
