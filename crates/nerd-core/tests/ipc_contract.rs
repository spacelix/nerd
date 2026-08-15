use jsonschema::validator_for;
use nerd_core::{
    APPLICATION_VERSION, IPC_PROTOCOL_VERSION,
    codec::{FRAME_PREFIX_BYTES, MAX_FRAME_BYTES, decode_payload, encode_frame},
    ipc::{
        ClientKind, DaemonHealth, DaemonIdentity, DataPaths, ErrorCode, ErrorResponse, Event,
        EventEnvelope, HandshakeRequest, HandshakeResponse, HealthComponent, HealthComponentName,
        HealthStatus, ProcessResources, ProgressEvent, Request, RequestEnvelope, Response,
        ResponseEnvelope, StatusRequest, StatusResponse,
    },
};
use serde_json::{Value, json};
use uuid::Uuid;

const ID: Uuid = Uuid::from_u128(0x12345678_1234_4234_8234_123456789abc);

#[test]
fn every_protocol_variant_matches_canonical_schema() {
    let schema: Value = serde_json::from_str(include_str!("../../../schemas/ipc.schema.json"))
        .expect("schema must be JSON");
    let validator = validator_for(&schema).expect("schema must compile");

    for fixture in fixtures() {
        let value = serde_json::to_value(fixture).expect("fixture must serialize");
        let errors: Vec<_> = validator
            .iter_errors(&value)
            .map(|error| error.to_string())
            .collect();
        assert!(errors.is_empty(), "schema errors for {value}: {errors:?}");
    }
}

#[test]
fn canonical_schema_rejects_unknown_fields() {
    let schema: Value = serde_json::from_str(include_str!("../../../schemas/ipc.schema.json"))
        .expect("schema must be JSON");
    let validator = validator_for(&schema).expect("schema must compile");
    let value = json!({
        "protocolVersion": 1,
        "requestId": ID,
        "request": {
            "type": "status",
            "payload": { "unexpected": true }
        }
    });
    assert!(!validator.is_valid(&value));

    let zero_based_request = RequestEnvelope {
        protocol_version: IPC_PROTOCOL_VERSION,
        request_id: ID,
        request: Request::Handshake(HandshakeRequest {
            client_kind: ClientKind::Cli,
            client_version: APPLICATION_VERSION.to_owned(),
            minimum_protocol_version: 0,
            maximum_protocol_version: IPC_PROTOCOL_VERSION,
        }),
    };
    let zero_based_range =
        serde_json::to_value(&zero_based_request).expect("serialize invalid range fixture");
    assert!(!validator.is_valid(&zero_based_range));
    assert!(encode_frame(&zero_based_request).is_err());

    let mismatch_without_range = json!({
        "protocolVersion": 1,
        "requestId": ID,
        "response": {
            "type": "error",
            "payload": {
                "code": "protocol_mismatch",
                "message": "incompatible",
                "retryable": false
            }
        }
    });
    assert!(!validator.is_valid(&mismatch_without_range));

    let unrelated_error_with_range = json!({
        "protocolVersion": 1,
        "requestId": ID,
        "response": {
            "type": "error",
            "payload": {
                "code": "invalid_request",
                "message": "invalid",
                "retryable": false,
                "minimumProtocolVersion": 1,
                "maximumProtocolVersion": 1
            }
        }
    });
    assert!(!validator.is_valid(&unrelated_error_with_range));
}

#[test]
fn serde_rejects_unknown_fields_at_each_object_boundary() {
    let top_level = json!({
        "protocolVersion": 1,
        "requestId": ID,
        "request": { "type": "status", "payload": {} },
        "unexpected": true
    });
    assert!(serde_json::from_value::<RequestEnvelope>(top_level).is_err());

    let nested = json!({
        "protocolVersion": 1,
        "requestId": ID,
        "request": { "type": "status", "payload": { "unexpected": true } }
    });
    assert!(serde_json::from_value::<RequestEnvelope>(nested).is_err());
}

#[test]
fn serde_rejects_values_outside_canonical_schema_constraints() {
    let zero_protocol = json!({
        "protocolVersion": 0,
        "requestId": ID,
        "request": { "type": "status", "payload": {} }
    });
    assert!(serde_json::from_value::<RequestEnvelope>(zero_protocol).is_err());

    let mismatch_without_range = json!({
        "protocolVersion": 1,
        "requestId": ID,
        "response": {
            "type": "error",
            "payload": {
                "code": "protocol_mismatch",
                "message": "incompatible",
                "retryable": false
            }
        }
    });
    assert!(serde_json::from_value::<ResponseEnvelope>(mismatch_without_range).is_err());

    let unrelated_error_with_range = json!({
        "protocolVersion": 1,
        "requestId": ID,
        "response": {
            "type": "error",
            "payload": {
                "code": "invalid_request",
                "message": "invalid",
                "retryable": false,
                "minimumProtocolVersion": 1,
                "maximumProtocolVersion": 1
            }
        }
    });
    assert!(serde_json::from_value::<ResponseEnvelope>(unrelated_error_with_range).is_err());

    let zero_sequence = json!({
        "protocolVersion": 1,
        "operationId": ID,
        "sequence": 0,
        "event": {
            "type": "progress",
            "payload": {
                "stage": "download",
                "message": "starting",
                "cancellable": true
            }
        }
    });
    assert!(serde_json::from_value::<EventEnvelope>(zero_sequence).is_err());
}

#[test]
fn codec_round_trips_and_enforces_frame_limit() {
    let request = handshake_request();
    let frame = encode_frame(&request).expect("request must encode");
    let length = u32::from_le_bytes(
        frame[..FRAME_PREFIX_BYTES]
            .try_into()
            .expect("prefix length"),
    ) as usize;
    assert_eq!(length, frame.len() - FRAME_PREFIX_BYTES);
    let decoded: RequestEnvelope =
        decode_payload(&frame[FRAME_PREFIX_BYTES..]).expect("request must decode");
    assert_eq!(decoded, request);

    let oversized = "x".repeat(MAX_FRAME_BYTES + 1);
    assert!(encode_frame(&oversized).is_err());
    assert!(decode_payload::<Value>(&vec![b' '; MAX_FRAME_BYTES + 1]).is_err());
}

fn fixtures() -> Vec<Value> {
    let mut fixtures = vec![
        serde_json::to_value(handshake_request()).expect("serialize handshake request"),
        serde_json::to_value(RequestEnvelope {
            protocol_version: IPC_PROTOCOL_VERSION,
            request_id: ID,
            request: Request::Status(StatusRequest {}),
        })
        .expect("serialize status request"),
        serde_json::to_value(ResponseEnvelope {
            protocol_version: IPC_PROTOCOL_VERSION,
            request_id: ID,
            response: Response::Handshake(HandshakeResponse {
                daemon_instance_id: ID,
                application_version: APPLICATION_VERSION.to_owned(),
                selected_protocol_version: IPC_PROTOCOL_VERSION,
            }),
        })
        .expect("serialize handshake response"),
        serde_json::to_value(ResponseEnvelope {
            protocol_version: IPC_PROTOCOL_VERSION,
            request_id: ID,
            response: Response::Status(status_response()),
        })
        .expect("serialize status response"),
        serde_json::to_value(ResponseEnvelope {
            protocol_version: IPC_PROTOCOL_VERSION,
            request_id: ID,
            response: Response::Error(ErrorResponse::protocol_mismatch()),
        })
        .expect("serialize protocol error"),
        serde_json::to_value(EventEnvelope {
            protocol_version: IPC_PROTOCOL_VERSION,
            operation_id: ID,
            sequence: 1,
            event: Event::Progress(ProgressEvent {
                stage: "download".to_owned(),
                message: "downloading artifact".to_owned(),
                cancellable: true,
                completed_units: Some(1),
                total_units: Some(2),
            }),
        })
        .expect("serialize event"),
    ];

    for code in [
        ErrorCode::HandshakeRequired,
        ErrorCode::InvalidRequest,
        ErrorCode::DaemonUnhealthy,
        ErrorCode::ShuttingDown,
        ErrorCode::Internal,
    ] {
        fixtures.push(
            serde_json::to_value(ResponseEnvelope {
                protocol_version: IPC_PROTOCOL_VERSION,
                request_id: ID,
                response: Response::Error(ErrorResponse::new(code, "request failed", false)),
            })
            .expect("serialize error response"),
        );
    }
    fixtures
}

fn handshake_request() -> RequestEnvelope {
    RequestEnvelope {
        protocol_version: IPC_PROTOCOL_VERSION,
        request_id: ID,
        request: Request::Handshake(HandshakeRequest {
            client_kind: ClientKind::Cli,
            client_version: APPLICATION_VERSION.to_owned(),
            minimum_protocol_version: IPC_PROTOCOL_VERSION,
            maximum_protocol_version: IPC_PROTOCOL_VERSION,
        }),
    }
}

fn status_response() -> StatusResponse {
    StatusResponse {
        daemon: DaemonIdentity {
            instance_id: ID,
            process_id: 42,
            application_version: APPLICATION_VERSION.to_owned(),
            protocol_version: IPC_PROTOCOL_VERSION,
            uptime_ms: 1_000,
        },
        health: DaemonHealth {
            status: HealthStatus::Degraded,
            components: vec![
                HealthComponent {
                    component: HealthComponentName::State,
                    status: HealthStatus::Healthy,
                    message: None,
                },
                HealthComponent {
                    component: HealthComponentName::Logging,
                    status: HealthStatus::Degraded,
                    message: Some("one event was dropped".to_owned()),
                },
                HealthComponent {
                    component: HealthComponentName::Ipc,
                    status: HealthStatus::Healthy,
                    message: None,
                },
                HealthComponent {
                    component: HealthComponentName::Resources,
                    status: HealthStatus::Unhealthy,
                    message: Some("metrics unavailable".to_owned()),
                },
            ],
        },
        paths: DataPaths {
            data_directory: r"C:\Users\dev\AppData\Local\Nerd".to_owned(),
            database_path: r"C:\Users\dev\AppData\Local\Nerd\nerd.db".to_owned(),
            log_directory: r"C:\Users\dev\AppData\Local\Nerd\logs".to_owned(),
        },
        resources: Some(ProcessResources {
            working_set_bytes: 1,
            peak_working_set_bytes: 2,
            private_usage_bytes: 3,
        }),
    }
}
