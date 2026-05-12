use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KisApiErrorBody {
    pub rt_cd: Option<String>,
    pub msg_cd: Option<String>,
    pub msg1: Option<String>,
}
