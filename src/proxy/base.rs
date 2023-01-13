use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SupportedProtocols_Dep {
    // SOCKS,
    TROJAN,
    DIRECT,
}
