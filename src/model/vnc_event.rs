pub enum VncEvent {
    Log(String),
    ConnectionError(String),
    Finished,
}
