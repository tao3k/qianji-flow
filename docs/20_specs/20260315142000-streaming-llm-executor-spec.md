---
id: "20260315142000"
type: knowledge
title: "Spec: Streaming LLM Executor (LlmAnalyzer) V1.1"
category: "architecture"
tags:
  - qianji
  - executor
  - llm-streaming
  - cognitive-trust
  - telemetry
saliency_base: 9.5
decay_rate: 0.01
metadata:
  title: "Spec: Streaming LLM Executor (LlmAnalyzer) V1.1"
---

# Spec: Streaming LLM Executor (LlmAnalyzer)

## 1. Overview

The **LlmAnalyzer** has been fully refactored into an active, **Streaming Cognitive Node**. It implements a high-precision token interception loop governed by the **ZhenfaPipeline**.

## 2. In-Flight Supervision Protocol (Physical)

The implementation enforces a "Zero-Trust" token loop:
1. **Live Stream**: Utilizes `client.chat_stream` for real-time token ingestion.
2. **Synthetic Normalization**: Every token chunk is transformed into a synthetic NDJSON line to maintain compatibility with the standard `Zhenfa` parsing logic.
3. **Interception**: Every line is validated by the **Logic Gate** (XSD) and scored by the **Cognitive Supervisor**.
4. **Early-Halt**: If `pipeline.should_halt()` returns true, the executor immediately issues a `FlowInstruction::Abort`, preventing token waste and logical drift.

## 3. Telemetry & Analytics

Every execution heartbeat is now observable:
- **CognitivePulse**: Emits real-time coherence scores and distribution metrics to the Swarm Telemetry Bus.
- **Data Density**: The final output includes a complete breakdown of `meta`, `operational`, and `epistemic` thought dimensions.

## 4. Implementation Reference

- **Kernel**: `xiuxian_zhenfa::ZhenfaPipeline`
- **File**: `packages/rust/crates/xiuxian-qianji/src/executors/llm/mechanism.rs`
- **Tests**: Comprehensive async streaming tests included in `mechanism.rs`.

---
## Linked Notes

- Parent MOC: [[20260315141000-qianji-orchestrator-moc]]
- Core Mechanism: [[packages/rust/crates/xiuxian-qianji/src/executors/llm/mechanism.rs]]
- Defense Engine: [[20260315154000-zhenfa-pipeline-spec]]
- Telemetry: [[packages/rust/crates/xiuxian-qianji/src/telemetry/events.rs]]
