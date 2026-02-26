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

use serde_json::json;

/// Sample Estimator V2 input payload for IBM backends
pub fn estimator_v2_input() -> String {
    json!({
        "pubs": [
            {
                "circuit": {
                    "num_qubits": 2,
                    "data": [
                        ["h", [0], []],
                        ["cx", [0, 1], []]
                    ]
                },
                "observables": [
                    {
                        "paulis": ["ZZ"],
                        "coeffs": [1.0]
                    }
                ],
                "precision": 0.01
            }
        ],
        "version": 2
    })
    .to_string()
}

/// Sample Sampler V2 input payload for IBM backends
pub fn sampler_v2_input() -> String {
    json!({
        "pubs": [
            {
                "circuit": {
                    "num_qubits": 2,
                    "data": [
                        ["h", [0], []],
                        ["cx", [0, 1], []],
                        ["measure", [0, 1], [0, 1]]
                    ]
                },
                "shots": 1024
            }
        ],
        "version": 2
    })
    .to_string()
}

/// Sample Noise Learner input payload for IBM backends
pub fn noise_learner_input() -> String {
    json!({
        "circuits": [
            {
                "num_qubits": 2,
                "data": [
                    ["h", [0], []],
                    ["cx", [0, 1], []]
                ]
            }
        ],
        "version": 1
    })
    .to_string()
}

/// Sample Pulser sequence for Pasqal backends
pub fn pulser_sequence() -> String {
    json!({
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
                    "pulses": [
                        {
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
                        }
                    ]
                }
            }
        }
    })
    .to_string()
}

/// Sample backend configuration for IBM Direct Access
pub fn backend_configuration() -> serde_json::Value {
    json!({
        "backend_name": "test_backend",
        "backend_version": "1.0.0",
        "n_qubits": 5,
        "basis_gates": ["id", "rz", "sx", "x", "cx"],
        "gates": [
            {
                "name": "cx",
                "parameters": [],
                "qasm_def": "gate cx q0,q1 { CX q0,q1; }",
                "coupling_map": [[0, 1], [1, 2], [2, 3], [3, 4]]
            }
        ],
        "local": false,
        "simulator": false,
        "conditional": false,
        "open_pulse": false,
        "memory": true,
        "max_shots": 100000,
        "coupling_map": [[0, 1], [1, 2], [2, 3], [3, 4]]
    })
}

/// Sample backend properties for IBM Direct Access
pub fn backend_properties() -> serde_json::Value {
    json!({
        "backend_name": "test_backend",
        "backend_version": "1.0.0",
        "last_update_date": "2025-02-25T12:00:00Z",
        "qubits": [
            [
                {"name": "T1", "unit": "µs", "value": 100.0},
                {"name": "T2", "unit": "µs", "value": 80.0},
                {"name": "frequency", "unit": "GHz", "value": 5.0},
                {"name": "readout_error", "unit": "", "value": 0.01}
            ]
        ],
        "gates": [
            {
                "gate": "cx",
                "qubits": [0, 1],
                "parameters": [
                    {"name": "gate_error", "unit": "", "value": 0.02},
                    {"name": "gate_length", "unit": "ns", "value": 200.0}
                ]
            }
        ],
        "general": []
    })
}

/// Sample job result for successful completion
pub fn job_result_success() -> serde_json::Value {
    json!({
        "results": [
            {
                "data": {
                    "evs": [0.95],
                    "stds": [0.01],
                    "ensemble_standard_error": 0.005
                },
                "metadata": {
                    "shots": 1024,
                    "target_precision": 0.01
                }
            }
        ],
        "metadata": {
            "version": "2.0"
        }
    })
}

/// Sample job logs
pub fn job_logs() -> String {
    "Job started at 2025-02-25T12:00:00Z\n\
     Compiling circuit...\n\
     Circuit compiled successfully\n\
     Executing on quantum hardware...\n\
     Job completed at 2025-02-25T12:05:00Z"
        .to_string()
}

/// Sample IAM token response
pub fn iam_token_response() -> serde_json::Value {
    json!({
        "access_token": "test_access_token_12345",
        "refresh_token": "test_refresh_token_67890",
        "token_type": "Bearer",
        "expires_in": 3600,
        "expiration": 1740484800,
        "scope": "ibm openid"
    })
}

/// Sample QRS session response
pub fn qrs_session_response(session_id: &str) -> serde_json::Value {
    json!({
        "id": session_id,
        "mode": "dedicated",
        "max_ttl": 28800,
        "state": "open",
        "timestamps": [
            {
                "status": "open",
                "timestamp": "2025-02-25T12:00:00Z"
            }
        ]
    })
}

/// Sample Pasqal device response
pub fn pasqal_device_response() -> serde_json::Value {
    json!({
        "id": "FRESNEL",
        "name": "Fresnel",
        "availability": "ACTIVE",
        "status": "AVAILABLE",
        "specs": {
            "max_atom_num": 100,
            "max_sequence_duration": 4000,
            "min_atom_distance": 4.0
        }
    })
}

/// Sample Pasqal batch response
pub fn pasqal_batch_response(batch_id: &str, status: &str) -> serde_json::Value {
    json!({
        "id": batch_id,
        "status": status,
        "device_type": "FRESNEL",
        "created_at": "2025-02-25T12:00:00Z",
        "updated_at": "2025-02-25T12:05:00Z",
        "jobs": [
            {
                "id": "job_1",
                "status": status
            }
        ]
    })
}

// Made with Bob
