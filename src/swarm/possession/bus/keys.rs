pub(super) fn request_key(request_id: &str) -> String {
    format!("xiuxian:swarm:possession:req:{request_id}")
}

pub(super) fn response_key(request_id: &str) -> String {
    format!("xiuxian:swarm:possession:resp:{request_id}")
}

pub(super) fn queue_key(role_class: &str) -> String {
    format!(
        "xiuxian:swarm:possession:queue:{}",
        role_class.trim().to_ascii_lowercase()
    )
}

pub(super) fn response_channel(request_id: &str) -> String {
    format!("xiuxian:swarm:possession:channel:{request_id}")
}
