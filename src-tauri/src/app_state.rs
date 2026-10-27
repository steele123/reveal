use crate::queue_pause::QueuePauseState;
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

pub struct QueuePause(pub Mutex<QueuePauseState>);

impl Default for QueuePause {
    fn default() -> Self {
        Self(Mutex::new(QueuePauseState::default()))
    }
}
