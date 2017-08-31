// Contains authentication information for a host.
#[derive(Clone)]
pub struct AuthKey {
    host: String,
    id: String,
    key: String,
    status: AuthKeyStatus,
}

// Status of an auth key (returns from /api/v1/auth/check).
#[derive(Clone)]
pub enum AuthKeyStatus {
    // Initial state of an AuthKey. No check has been done yet.
    None,
    // User pressed "Accept" on the printer.
    Authorized,
    // User pressed "Reject" on the printer, or expired (printer restart, etc)
    Unauthorized,
    // Waiting on user input on printer (Accept/Reject dialog is open).
    Unknown,
}

impl AuthKey {
    pub fn new (host: String, id: String, key: String) -> AuthKey {
        AuthKey {
            host: host,
            id: id,
            key: key,
            status: AuthKeyStatus::None,
        }
    }

    // Getter Functions
    pub fn host (&self) -> &String {
        &self.host
    }

    pub fn id (&self) -> &String {
        &self.id
    }

    // pub fn key (&self) -> &String {
    //     &self.key
    // }

    // pub fn status (&self) -> &AuthKeyStatus {
    //     &self.status
    // }

    // Mutable Getter Functions
    // pub fn host_mut (&mut self) -> &mut String {
    //     &mut self.host
    // }

    // pub fn id_mut (&mut self) -> &mut String {
    //     &mut self.id
    // }

    // pub fn key_mut (&mut self) -> &mut String {
    //     &mut self.key
    // }

    // pub fn status_mut (&mut self) -> &mut AuthKeyStatus {
    //     &mut self.status
    // }
}