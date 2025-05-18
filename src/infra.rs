use tokio::sync::mpsc::{Receiver, Sender, channel};

#[derive(Default, Clone, Debug)]
pub struct QueueService {}

#[allow(dead_code)]
impl QueueService {
    pub fn channel<T>(&self, _name: &str) -> (Sender<T>, Receiver<T>) {
        channel(10)
    }
}

#[allow(dead_code)]
pub mod task_queue {
    use crate::logic::ObsPackage;
    pub use tokio::sync::mpsc;
    pub type Sender = mpsc::Sender<ObsPackage>;
    pub type Receiver = mpsc::Receiver<ObsPackage>;
}

#[allow(dead_code)]
pub mod raw_queue {
    pub use tokio::sync::mpsc;
    pub type Sender = mpsc::Sender<String>;
    pub type Receiver = mpsc::Receiver<String>;
}

#[allow(dead_code)]
pub mod data_queue {
    pub use tokio::sync::mpsc;
    pub type Sender = mpsc::Sender<String>;
    pub type Receiver = mpsc::Receiver<String>;
}

#[allow(dead_code)]
pub(crate) mod warn_queue {
    use crate::logic::TrapRecord;
    pub(crate) type Sender = tokio::sync::mpsc::Sender<TrapRecord>;
    pub(crate) type Receiver = tokio::sync::mpsc::Receiver<TrapRecord>;
}
