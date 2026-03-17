---
id: "20260315140000"
type: knowledge
title: "Feature: Autonomous Audit Scenario V3.0"
category: "features"
tags:
  - qianji
  - audit
  - trinity-loop
  - automation
saliency_base: 8.5
decay_rate: 0.01
metadata:
  title: "Feature: Autonomous Audit Scenario V3.0"
---

# Feature: Autonomous Audit Scenario V3.0 (UNATTENDED)

## 1. Overview

This feature implements the **"Triple Loop"** (Plan → Execute → Verify) as a native **Qianji LinkGraph** pattern. Unlike traditional hardcoded scripts, the audit logic is fully declarative and resides in `audit_flow.toml`.

## 2. Key Capabilities

- **Native Edge Remediation**: If the `verify` node fails, the graph automatically loops back to the `execute` node via a conditional edge.
- **Context-Aware Retries**: Maintains `remediation_attempts` in the execution context to prevent infinite loops and provide feedback to the AI.

---

## Linked Notes

- Parent MOC: [[20260315141000-qianji-orchestrator-moc]]
- Contract: [[docs/assets/schemas/audit/qianji_plan.xsd]] (Ref)
- Theory: [[20260315140500-autonomous-engineering-foundations]]
