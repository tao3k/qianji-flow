use super::QianjiScheduler;
use crate::contracts::NodeStatus;
use crate::scheduler::checkpoint::QianjiStateSnapshot;
use crate::scheduler::state::merge_output_data;
use petgraph::stable_graph::NodeIndex;
use std::collections::{HashMap, HashSet};

impl QianjiScheduler {
    pub(super) async fn reset_retry_nodes(&self, node_ids: &[String]) {
        let mut engine = self.engine.write().await;
        let mut to_reset = HashSet::new();

        let initial_indices: Vec<_> = engine
            .graph
            .node_indices()
            .filter(|&idx| node_ids.contains(&engine.graph[idx].id))
            .collect();

        for start_idx in initial_indices {
            let mut bfs = petgraph::visit::Bfs::new(&engine.graph, start_idx);
            while let Some(visited) = bfs.next(&engine.graph) {
                to_reset.insert(visited);
            }
        }

        for idx in to_reset {
            engine.graph[idx].status = NodeStatus::Idle;
        }
    }

    pub(super) async fn apply_snapshot_node_statuses(
        &self,
        snapshot: &QianjiStateSnapshot,
    ) -> bool {
        let mut changed = false;
        let mut engine = self.engine.write().await;
        let indices: Vec<_> = engine.graph.node_indices().collect();
        for node_idx in indices {
            let id = engine.graph[node_idx].id.clone();
            if let Some(status) = snapshot.node_statuses.get(&id)
                && engine.graph[node_idx].status != *status
            {
                engine.graph[node_idx].status = status.clone();
                changed = true;
            }
        }
        changed
    }

    pub(super) async fn load_checkpoint_state(
        &self,
        initial_context: &serde_json::Value,
        session_id: Option<&str>,
        redis_url: Option<&str>,
    ) -> (serde_json::Value, HashSet<String>, u32) {
        let mut context = initial_context.clone();
        let mut active_branches: HashSet<String> = HashSet::new();
        let mut total_steps = 0;

        let (Some(sid), Some(url)) = (session_id, redis_url) else {
            return (context, active_branches, total_steps);
        };

        match QianjiStateSnapshot::load(sid, url).await {
            Ok(Some(snapshot)) => {
                let mut merged_context = snapshot.context.clone();
                merge_output_data(&mut merged_context, initial_context);
                context = merged_context;
                active_branches = snapshot.active_branches.clone();
                total_steps = snapshot.total_steps;
                let _ = self.apply_snapshot_node_statuses(&snapshot).await;
            }
            Ok(None) => {}
            Err(error) => {
                log::warn!("Failed to load checkpoint for session {sid}: {error}. Starting fresh.");
            }
        }

        (context, active_branches, total_steps)
    }

    pub(super) async fn save_checkpoint_if_needed(
        &self,
        session_id: Option<&str>,
        redis_url: Option<&str>,
        total_steps: u32,
        active_branches: &HashSet<String>,
        context: &serde_json::Value,
    ) {
        let (Some(sid), Some(url)) = (session_id, redis_url) else {
            return;
        };

        let engine = self.engine.read().await;
        let mut node_statuses = HashMap::new();
        for node_idx in engine.graph.node_indices() {
            let node = &engine.graph[node_idx];
            node_statuses.insert(node.id.clone(), node.status.clone());
        }

        let snapshot = QianjiStateSnapshot {
            session_id: sid.to_string(),
            total_steps,
            active_branches: active_branches.clone(),
            context: context.clone(),
            node_statuses,
        };

        if let Err(error) = snapshot.save(url).await {
            log::warn!("Failed to save checkpoint for session {sid}: {error}");
        }
    }

    pub(super) async fn delete_checkpoint_if_needed(
        &self,
        session_id: Option<&str>,
        redis_url: Option<&str>,
    ) {
        let (Some(sid), Some(url)) = (session_id, redis_url) else {
            return;
        };
        let _ = QianjiStateSnapshot::delete(sid, url).await;
    }

    pub(super) async fn set_node_status(&self, node_idx: NodeIndex, status: NodeStatus) {
        let mut engine = self.engine.write().await;
        engine.graph[node_idx].status = status;
    }

    pub(super) async fn node_ids_for_indices(&self, nodes: &[NodeIndex]) -> Vec<String> {
        let engine = self.engine.read().await;
        nodes
            .iter()
            .filter_map(|index| engine.graph.node_weight(*index).map(|node| node.id.clone()))
            .collect()
    }
}
