use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

const TTL: Duration = Duration::from_secs(10 * 60);

#[derive(Clone)]
pub struct PendingSignup {
    pub handle: String,
    pub email: String,
    pub password: String,
    pub invite_code: String,
    expires_at: Instant,
}

impl PendingSignup {
    pub fn new(handle: String, email: String, password: String, invite_code: String) -> Self {
        Self {
            handle,
            email,
            password,
            invite_code,
            expires_at: Instant::now() + TTL,
        }
    }
}

#[derive(Default)]
pub struct SignupSessions {
    inner: Mutex<HashMap<String, PendingSignup>>,
}

impl SignupSessions {
    pub fn save(&self, state: String, pending: PendingSignup) {
        let mut map = self.inner.lock().expect("sessions mutex poisoned");
        let now = Instant::now();
        map.retain(|_, v| v.expires_at > now);
        map.insert(state, pending);
    }

    pub fn take(&self, state: &str) -> Option<PendingSignup> {
        let mut map = self.inner.lock().expect("sessions mutex poisoned");
        let entry = map.remove(state)?;
        if entry.expires_at <= Instant::now() {
            None
        } else {
            Some(entry)
        }
    }
}
