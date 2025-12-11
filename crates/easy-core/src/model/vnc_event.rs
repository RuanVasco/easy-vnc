use super::VncError;

pub enum VncEvent {
    Log(String),
    ConnectionError(VncError),
    Finished,
}
