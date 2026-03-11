use crate::error::QianjiError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TaskType {
    Knowledge,
    Annotation,
    Calibration,
    FormalAudit,
    Llm,
    Mock,
    Command,
    WriteFile,
    Suspend,
    SecurityScan,
    WendaoIngester,
    WendaoRefresh,
    Router,
}

impl TaskType {
    pub(super) fn parse(raw: &str) -> Result<Self, QianjiError> {
        match raw {
            "knowledge" => Ok(Self::Knowledge),
            "annotation" => Ok(Self::Annotation),
            "calibration" => Ok(Self::Calibration),
            "formal_audit" => Ok(Self::FormalAudit),
            "llm" => Ok(Self::Llm),
            "mock" => Ok(Self::Mock),
            "command" => Ok(Self::Command),
            "write_file" => Ok(Self::WriteFile),
            "suspend" => Ok(Self::Suspend),
            "security_scan" => Ok(Self::SecurityScan),
            "wendao_ingester" => Ok(Self::WendaoIngester),
            "wendao_refresh" => Ok(Self::WendaoRefresh),
            "router" => Ok(Self::Router),
            _ => Err(QianjiError::Topology(format!("Unknown task type: {raw}"))),
        }
    }
}
