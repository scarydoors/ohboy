pub enum RequestedIMEState {
    Enable,
    Disable,
}

pub struct Interrupts {
    // interrupt master enable flag
    ime: bool,
    ime_requested_state: Option<RequestedIMEState>,
}
