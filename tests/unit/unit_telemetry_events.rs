//! Unit tests for telemetry events including CognitivePulse.

use serde_json;
use xiuxian_qianji::telemetry::{
    CognitiveDistributionMetrics, ConsensusStatus, NodeTransitionPhase, SwarmEvent,
    DEFAULT_PULSE_CHANNEL, unix_millis_now,
};

#[test]
fn swarm_event_cognitive_pulse_serializes_correctly() {
    let event = SwarmEvent::CognitivePulse {
        session_id: Some("test-session-123".to_string()),
        node_id: "audit_node_1".to_string(),
        coherence: 0.85,
        early_halt_triggered: false,
        distribution: CognitiveDistributionMetrics {
            meta: 0.7,
            operational: 0.8,
            epistemic: 0.75,
            instrumental: 0.82,
            balance: 0.76,
            uncertainty_ratio: 0.15,
        },
        timestamp_ms: 1700000000000,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"event\":\"cognitive_pulse\""));
    assert!(json.contains("\"session_id\":\"test-session-123\""));
    assert!(json.contains("\"node_id\":\"audit_node_1\""));
    assert!(json.contains("\"coherence\":0.85"));
    assert!(json.contains("\"early_halt_triggered\":false"));
    assert!(json.contains("\"meta\":0.7"));
    assert!(json.contains("\"operational\":0.8"));
}

#[test]
fn swarm_event_cognitive_pulse_deserializes_correctly() {
    let json = r#"{
        "event": "cognitive_pulse",
        "session_id": "session-456",
        "node_id": "node-789",
        "coherence": 0.42,
        "early_halt_triggered": true,
        "distribution": {
            "meta": 0.3,
            "operational": 0.4,
            "epistemic": 0.35,
            "instrumental": 0.45,
            "balance": 0.375,
            "uncertainty_ratio": 0.6
        },
        "timestamp_ms": 1700000001000
    }"#;

    let event: SwarmEvent = serde_json::from_str(json).unwrap();

    match event {
        SwarmEvent::CognitivePulse {
            session_id,
            node_id,
            coherence,
            early_halt_triggered,
            distribution,
            timestamp_ms,
        } => {
            assert_eq!(session_id, Some("session-456".to_string()));
            assert_eq!(node_id, "node-789");
            assert!((coherence - 0.42).abs() < 1e-6);
            assert!(early_halt_triggered);
            assert!((distribution.meta - 0.3).abs() < 1e-6);
            assert!((distribution.operational - 0.4).abs() < 1e-6);
            assert!((distribution.epistemic - 0.35).abs() < 1e-6);
            assert!((distribution.instrumental - 0.45).abs() < 1e-6);
            assert!((distribution.balance - 0.375).abs() < 1e-6);
            assert!((distribution.uncertainty_ratio - 0.6).abs() < 1e-6);
            assert_eq!(timestamp_ms, 1700000001000);
        }
        _ => panic!("Expected CognitivePulse variant"),
    }
}

#[test]
fn cognitive_distribution_metrics_serialization_roundtrip() {
    let metrics = CognitiveDistributionMetrics {
        meta: 0.5,
        operational: 0.6,
        epistemic: 0.55,
        instrumental: 0.58,
        balance: 0.56,
        uncertainty_ratio: 0.25,
    };

    let json = serde_json::to_string(&metrics).unwrap();
    let deserialized: CognitiveDistributionMetrics = serde_json::from_str(&json).unwrap();

    assert!((deserialized.meta - metrics.meta).abs() < 1e-6);
    assert!((deserialized.operational - metrics.operational).abs() < 1e-6);
    assert!((deserialized.epistemic - metrics.epistemic).abs() < 1e-6);
    assert!((deserialized.instrumental - metrics.instrumental).abs() < 1e-6);
    assert!((deserialized.balance - metrics.balance).abs() < 1e-6);
    assert!((deserialized.uncertainty_ratio - metrics.uncertainty_ratio).abs() < 1e-6);
}

#[test]
fn swarm_event_node_transition_serialization() {
    let event = SwarmEvent::NodeTransition {
        session_id: Some("session-123".to_string()),
        agent_id: Some("agent-456".to_string()),
        role_class: Some("auditor".to_string()),
        node_id: "node-789".to_string(),
        phase: NodeTransitionPhase::Entering,
        timestamp_ms: 1700000002000,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"event\":\"node_transition\""));
    assert!(json.contains("\"phase\":\"entering\""));
}

#[test]
fn swarm_event_consensus_spike_serialization() {
    let event = SwarmEvent::ConsensusSpike {
        session_id: "session-123".to_string(),
        node_id: "node-456".to_string(),
        status: ConsensusStatus::Pending,
        progress: Some(0.6),
        target: Some(1.0),
        timestamp_ms: 1700000003000,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"event\":\"consensus_spike\""));
    assert!(json.contains("\"status\":\"pending\""));
    assert!(json.contains("\"progress\":0.6"));
}

#[test]
fn swarm_event_evolution_birth_serialization() {
    let event = SwarmEvent::EvolutionBirth {
        session_id: Some("session-123".to_string()),
        role_id: Some("steward".to_string()),
        manifestation_path: "/output/artifact.md".to_string(),
        timestamp_ms: 1700000004000,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"event\":\"evolution_birth\""));
    assert!(json.contains("\"manifestation_path\":\"/output/artifact.md\""));
}

#[test]
fn swarm_event_affinity_alert_serialization() {
    let event = SwarmEvent::AffinityAlert {
        session_id: Some("session-123".to_string()),
        node_id: "node-456".to_string(),
        required_role: "specialist".to_string(),
        proxy_agent_id: Some("proxy-agent-789".to_string()),
        proxy_role: Some("generalist".to_string()),
        timestamp_ms: 1700000005000,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"event\":\"affinity_alert\""));
    assert!(json.contains("\"required_role\":\"specialist\""));
}

#[test]
fn swarm_event_heartbeat_serialization() {
    let event = SwarmEvent::SwarmHeartbeat {
        session_id: Some("session-123".to_string()),
        cluster_id: Some("cluster-456".to_string()),
        agent_id: Some("agent-789".to_string()),
        role_class: Some("worker".to_string()),
        cpu_percent: Some(45.5),
        memory_bytes: Some(1024 * 1024 * 256),
        timestamp_ms: 1700000006000,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"event\":\"swarm_heartbeat\""));
    assert!(json.contains("\"cpu_percent\":45.5"));
}

#[test]
fn node_transition_phase_variants() {
    assert_eq!(
        serde_json::to_string(&NodeTransitionPhase::Entering).unwrap(),
        "\"entering\""
    );
    assert_eq!(
        serde_json::to_string(&NodeTransitionPhase::Exiting).unwrap(),
        "\"exiting\""
    );
    assert_eq!(
        serde_json::to_string(&NodeTransitionPhase::Failed).unwrap(),
        "\"failed\""
    );
}

#[test]
fn consensus_status_variants() {
    assert_eq!(
        serde_json::to_string(&ConsensusStatus::Pending).unwrap(),
        "\"pending\""
    );
    assert_eq!(
        serde_json::to_string(&ConsensusStatus::Agreed).unwrap(),
        "\"agreed\""
    );
    assert_eq!(
        serde_json::to_string(&ConsensusStatus::Failed).unwrap(),
        "\"failed\""
    );
}

#[test]
fn default_pulse_channel_value() {
    assert_eq!(DEFAULT_PULSE_CHANNEL, "xiuxian:swarm:pulse");
}

#[test]
fn unix_millis_now_returns_reasonable_value() {
    let now = unix_millis_now();
    // Should be after 2024-01-01 and before 2100-01-01
    assert!(now > 1704067200000); // 2024-01-01
    assert!(now < 4102444800000); // 2100-01-01
}

#[test]
fn unix_millis_now_monotonic_increase() {
    let t1 = unix_millis_now();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let t2 = unix_millis_now();
    assert!(t2 >= t1, "timestamp should not decrease");
}

#[test]
fn cognitive_pulse_with_none_session_id() {
    let event = SwarmEvent::CognitivePulse {
        session_id: None,
        node_id: "test-node".to_string(),
        coherence: 0.5,
        early_halt_triggered: false,
        distribution: CognitiveDistributionMetrics {
            meta: 0.5,
            operational: 0.5,
            epistemic: 0.5,
            instrumental: 0.5,
            balance: 0.5,
            uncertainty_ratio: 0.5,
        },
        timestamp_ms: 1700000000000,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"session_id\":null"));

    let deserialized: SwarmEvent = serde_json::from_str(&json).unwrap();
    match deserialized {
        SwarmEvent::CognitivePulse { session_id, .. } => {
            assert!(session_id.is_none());
        }
        _ => panic!("Expected CognitivePulse"),
    }
}
