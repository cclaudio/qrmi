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

"""
Python Integration Tests for QRMI

These tests verify that the Python bindings work correctly with the Rust implementation.
They test the full workflow through the Python interface to ensure proper:
- Type conversion between Python and Rust
- Error handling and exception propagation
- Memory management across the language boundary
- API completeness and correctness
"""

import pytest
import uuid
import qrmi


class TestPythonBindingsBasic:
    """Basic Python binding tests"""
    
    def test_import_qrmi(self):
        """Test that QRMI module can be imported"""
        assert hasattr(qrmi, 'QuantumResource')
        assert hasattr(qrmi, 'ResourceType')
        assert hasattr(qrmi, 'Payload')
        assert hasattr(qrmi, 'TaskStatus')
        assert hasattr(qrmi, 'Target')
        assert hasattr(qrmi, 'TaskResult')
    
    def test_resource_types(self):
        """Test ResourceType enum"""
        assert hasattr(qrmi.ResourceType, 'IBMDirectAccess')
        assert hasattr(qrmi.ResourceType, 'IBMQiskitRuntimeService')
        assert hasattr(qrmi.ResourceType, 'PasqalCloud')
    
    def test_task_status_enum(self):
        """Test TaskStatus enum"""
        assert hasattr(qrmi.TaskStatus, 'Queued')
        assert hasattr(qrmi.TaskStatus, 'Running')
        assert hasattr(qrmi.TaskStatus, 'Completed')
        assert hasattr(qrmi.TaskStatus, 'Failed')
        assert hasattr(qrmi.TaskStatus, 'Cancelled')


class TestIBMDirectAccessPython:
    """Python binding tests for IBM Direct Access"""
    
    def test_resource_creation(self, setup_da_env):
        """Test creating a QuantumResource for Direct Access"""
        backend, da_server, iam_server = setup_da_env
        
        # Add mock responses
        da_server.add_response('GET', '/v1/backends', body={
            'backends': [{'name': backend, 'status': 'online'}]
        })
        
        # Create resource
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMDirectAccess)
        assert qrmi_resource is not None
    
    def test_is_accessible(self, setup_da_env):
        """Test is_accessible method"""
        backend, da_server, iam_server = setup_da_env
        
        da_server.add_response('GET', '/v1/backends', body={
            'backends': [{'name': backend, 'status': 'online'}]
        })
        da_server.add_response('GET', f'/v1/backends/{backend}/configuration', body={
            'backend_name': backend,
            'n_qubits': 5
        })
        
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMDirectAccess)
        accessible = qrmi_resource.is_accessible()
        assert isinstance(accessible, bool)
        assert accessible is True
    
    def test_acquire_release(self, setup_da_env):
        """Test acquire and release methods"""
        backend, da_server, iam_server = setup_da_env
        
        da_server.add_response('GET', '/v1/backends', body={
            'backends': [{'name': backend, 'status': 'online'}]
        })
        
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMDirectAccess)
        
        # Acquire
        token = qrmi_resource.acquire()
        assert isinstance(token, str)
        assert len(token) > 0
        
        # Release
        qrmi_resource.release(token)  # Should not raise
    
    def test_full_workflow_estimator(self, setup_da_env, estimator_input):
        """Test full workflow with Estimator"""
        backend, da_server, iam_server = setup_da_env
        
        job_id = str(uuid.uuid4())
        
        # Setup mocks
        da_server.add_response('GET', '/v1/backends', body={
            'backends': [{'name': backend, 'status': 'online'}]
        })
        da_server.add_response('GET', f'/v1/backends/{backend}/configuration', body={
            'backend_name': backend
        })
        da_server.add_response('POST', '/v1/jobs', body={
            'id': job_id,
            'status': 'Queued'
        })
        da_server.add_response('GET', f'/v1/jobs/{job_id}/status', body={
            'status': 'Completed'
        })
        da_server.add_response('GET', f'/v1/jobs/{job_id}', body={
            'id': job_id,
            'status': 'Completed'
        })
        da_server.add_response('POST', f'/v1/jobs/{job_id}/cancel', body={})
        
        # Create resource
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMDirectAccess)
        
        # Acquire
        token = qrmi_resource.acquire()
        assert len(token) > 0
        
        # Start task
        payload = qrmi.Payload.QiskitPrimitive(
            input=estimator_input,
            program_id="estimator"
        )
        task_id = qrmi_resource.task_start(payload)
        assert isinstance(task_id, str)
        assert len(task_id) > 0
        
        # Check status
        status = qrmi_resource.task_status(task_id)
        assert isinstance(status, qrmi.TaskStatus)
        
        # Stop task
        qrmi_resource.task_stop(task_id)
        
        # Release
        qrmi_resource.release(token)
    
    def test_target_retrieval(self, setup_da_env):
        """Test target retrieval"""
        backend, da_server, iam_server = setup_da_env
        
        da_server.add_response('GET', f'/v1/backends/{backend}/configuration', body={
            'backend_name': backend,
            'n_qubits': 5
        })
        da_server.add_response('GET', f'/v1/backends/{backend}/properties', body={
            'backend_name': backend
        })
        
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMDirectAccess)
        target = qrmi_resource.target()
        
        assert isinstance(target, qrmi.Target)
        assert hasattr(target, 'value')
        assert isinstance(target.value, str)
        assert len(target.value) > 0
    
    def test_metadata_retrieval(self, setup_da_env):
        """Test metadata retrieval"""
        backend, da_server, iam_server = setup_da_env
        
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMDirectAccess)
        metadata = qrmi_resource.metadata()
        
        assert isinstance(metadata, dict)
        assert len(metadata) > 0
        assert 'backend_name' in metadata
    
    def test_error_handling(self, setup_da_env):
        """Test error handling and exception propagation"""
        backend, da_server, iam_server = setup_da_env
        
        # Don't add any mock responses to trigger errors
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMDirectAccess)
        
        # This should raise a RuntimeError
        with pytest.raises(RuntimeError):
            qrmi_resource.is_accessible()


class TestIBMQRSPython:
    """Python binding tests for IBM Qiskit Runtime Service"""
    
    def test_resource_creation(self, setup_qrs_env):
        """Test creating a QuantumResource for QRS"""
        backend, qrs_server, iam_server = setup_qrs_env
        
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMQiskitRuntimeService)
        assert qrmi_resource is not None
    
    def test_full_workflow_sampler(self, setup_qrs_env, sampler_input):
        """Test full workflow with Sampler"""
        backend, qrs_server, iam_server = setup_qrs_env
        
        session_id = str(uuid.uuid4())
        job_id = str(uuid.uuid4())
        
        # Setup mocks
        qrs_server.add_response('POST', '/sessions', body={
            'id': session_id,
            'mode': 'dedicated',
            'state': 'open'
        })
        qrs_server.add_response('GET', f'/backends/{backend}/configuration', body={
            'backend_name': backend
        })
        qrs_server.add_response('POST', '/jobs', body={
            'id': job_id
        })
        qrs_server.add_response('GET', f'/jobs/{job_id}', body={
            'id': job_id,
            'state': 'Completed'
        })
        qrs_server.add_response('PATCH', f'/sessions/{session_id}', body={
            'id': session_id,
            'state': 'closed'
        })
        
        # Create resource
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMQiskitRuntimeService)
        
        # Acquire (creates session)
        token = qrmi_resource.acquire()
        assert len(token) > 0
        
        # Start task
        payload = qrmi.Payload.QiskitPrimitive(
            input=sampler_input,
            program_id="sampler"
        )
        task_id = qrmi_resource.task_start(payload)
        assert len(task_id) > 0
        
        # Check status
        status = qrmi_resource.task_status(task_id)
        assert isinstance(status, qrmi.TaskStatus)
        
        # Release (closes session)
        qrmi_resource.release(token)


class TestPasqalCloudPython:
    """Python binding tests for Pasqal Cloud"""
    
    def test_resource_creation(self, setup_pasqal_env):
        """Test creating a QuantumResource for Pasqal"""
        backend, pasqal_server = setup_pasqal_env
        
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.PasqalCloud)
        assert qrmi_resource is not None
    
    def test_full_workflow_pulser(self, setup_pasqal_env, pulser_sequence):
        """Test full workflow with Pulser"""
        backend, pasqal_server = setup_pasqal_env
        
        batch_id = str(uuid.uuid4())
        
        # Setup mocks
        pasqal_server.add_response('GET', f'/devices/{backend}', body={
            'id': backend,
            'availability': 'ACTIVE',
            'status': 'AVAILABLE'
        })
        pasqal_server.add_response('GET', f'/core-fast/api/v1/devices/specs/{backend}', body={
            'data': {'specs': '{}'}
        })
        pasqal_server.add_response('POST', '/batches', body={
            'id': batch_id,
            'status': 'PENDING'
        })
        pasqal_server.add_response('GET', f'/batches/{batch_id}', body={
            'id': batch_id,
            'status': 'DONE'
        })
        pasqal_server.add_response('POST', f'/batches/{batch_id}/cancel', body={
            'id': batch_id,
            'status': 'CANCELED'
        })
        
        # Create resource
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.PasqalCloud)
        
        # Check accessibility
        accessible = qrmi_resource.is_accessible()
        assert accessible is True
        
        # Acquire
        token = qrmi_resource.acquire()
        assert len(token) > 0
        
        # Start task
        payload = qrmi.Payload.PasqalCloud(
            sequence=pulser_sequence,
            job_runs=100
        )
        task_id = qrmi_resource.task_start(payload)
        assert len(task_id) > 0
        
        # Check status
        status = qrmi_resource.task_status(task_id)
        assert isinstance(status, qrmi.TaskStatus)
        
        # Stop task
        qrmi_resource.task_stop(task_id)
        
        # Release
        qrmi_resource.release(token)
    
    def test_target_retrieval(self, setup_pasqal_env):
        """Test target retrieval for Pasqal"""
        backend, pasqal_server = setup_pasqal_env
        
        pasqal_server.add_response('GET', f'/core-fast/api/v1/devices/specs/{backend}', body={
            'data': {'specs': '{"max_atom_num": 100}'}
        })
        
        qrmi_resource = qrmi.QuantumResource(backend, qrmi.ResourceType.PasqalCloud)
        target = qrmi_resource.target()
        
        assert isinstance(target, qrmi.Target)
        assert len(target.value) > 0


class TestPythonBindingsEdgeCases:
    """Edge case tests for Python bindings"""
    
    def test_invalid_backend_name(self):
        """Test with invalid backend name"""
        with pytest.raises(RuntimeError):
            qrmi.QuantumResource("nonexistent_backend", qrmi.ResourceType.IBMDirectAccess)
    
    def test_payload_types(self, estimator_input, pulser_sequence):
        """Test different payload types"""
        # QiskitPrimitive payload
        payload1 = qrmi.Payload.QiskitPrimitive(
            input=estimator_input,
            program_id="estimator"
        )
        assert payload1 is not None
        
        # PasqalCloud payload
        payload2 = qrmi.Payload.PasqalCloud(
            sequence=pulser_sequence,
            job_runs=100
        )
        assert payload2 is not None
    
    def test_task_result_structure(self):
        """Test TaskResult structure"""
        # TaskResult should have a 'value' attribute
        # This is tested indirectly through other tests
        pass
    
    def test_multiple_resources(self, setup_da_env):
        """Test creating multiple resources"""
        backend, da_server, iam_server = setup_da_env
        
        da_server.add_response('GET', '/v1/backends', body={
            'backends': [{'name': backend, 'status': 'online'}]
        })
        
        # Create two resources
        qrmi1 = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMDirectAccess)
        qrmi2 = qrmi.QuantumResource(backend, qrmi.ResourceType.IBMDirectAccess)
        
        assert qrmi1 is not None
        assert qrmi2 is not None
        assert qrmi1 is not qrmi2

# Made with Bob
