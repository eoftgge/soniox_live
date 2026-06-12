#[derive(PartialEq, Eq)]
pub enum StreamAction {
    Continue,
    Reconnect,
    Stop,
}
