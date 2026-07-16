use shaco::rest::LCUClientInfo;
use tokio::sync::Mutex;

pub struct Lcu(pub Mutex<LcuConnection>);

pub struct LcuConnection {
    pub connected: bool,
    pub data: Option<LCUClientInfo>,
}

impl Default for Lcu {
    fn default() -> Self {
        Self(Mutex::new(LcuConnection {
            connected: false,
            data: None,
        }))
    }
}

pub struct Dodge(pub Mutex<DodgeState>);

#[derive(Default)]
pub struct DodgeState {
    pub last_dodge: Option<u64>,
    pub enabled: Option<u64>,
}

impl Default for Dodge {
    fn default() -> Self {
        Self(Mutex::new(DodgeState::default()))
    }
}
